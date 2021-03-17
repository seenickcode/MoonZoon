use wasm_bindgen::{closure::Closure, JsCast};
use crate::{RenderContext, dom::dom_element, Node, Element, IntoElement, ApplyToElement, render, element_macro};
use crate::hook::l_var;
use crate::l_var::LVar;
use crate::runtime::rerender;
use std::{cell::RefCell, rc::Rc};

// ------ ------
//    Element 
// ------ ------

element_macro!(button, Button::default());

#[derive(Default)]
pub struct Button<'a> {
    label: Option<Box<dyn Element + 'a>>,
    on_press: Option<OnPress>,
}

impl<'a> Element for Button<'a> {
    #[render]
    fn render(&mut self, rcx: RenderContext ) {
        // log!("button, index: {}", rcx.index);

        let node = dom_element(rcx, |rcx: RenderContext| {
            if let Some(label) = self.label.as_mut() {
                label.render(rcx);
            }
        });
        node.update_mut(|node| {
            let element = node.node_ws.unchecked_ref::<web_sys::Element>();
            element.set_attribute("class", "button").unwrap();
            element.set_attribute("role", "button").unwrap();
            element.set_attribute("tabindex", "0").unwrap();
        });

        if let Some(OnPress(on_press)) = self.on_press.take() {
            let listener = l_var(|| Listener::new("click", node));
            listener.update_mut(|listener| listener.set_handler(on_press));
        }
    }
}

struct Listener {
    event: &'static str,
    node: LVar<Node>,
    handler: Rc<RefCell<Box<dyn Fn()>>>,
    callback: Closure<dyn Fn()>,
}

impl Listener {
    fn new(event: &'static str, node: LVar<Node>) -> Self {
        let dummy_handler = Box::new(||()) as Box<dyn Fn()>;
        let handler = Rc::new(RefCell::new(dummy_handler));

        let handler_clone = Rc::clone(&handler);
        let callback = Closure::wrap(Box::new(move || {
            handler_clone.borrow()();
            rerender();
        }) as Box<dyn Fn()>);

        node.update_mut(|node| {
            node
                .node_ws
                .unchecked_ref::<web_sys::EventTarget>()
                .add_event_listener_with_callback(event, callback.as_ref().unchecked_ref())
                .expect("add event listener");
        });

        Self {
            event,
            node,
            handler,
            callback,
        }
    }

    fn set_handler(&mut self, handler: Box<dyn Fn()>) {
        *self.handler.borrow_mut() = handler;
    }
}

impl Drop for Listener{
    fn drop(&mut self) {
        if !self.node.exists() {
            return;
        }
        self.node.update_mut(|node| {
            node
                .node_ws
                .unchecked_ref::<web_sys::EventTarget>()
                .remove_event_listener_with_callback(
                    self.event,
                    self.callback.as_ref().unchecked_ref(),
                )
                .expect("remove event listener");
        });
    }
}

// ------ ------
//  Attributes 
// ------ ------

// ------ IntoElement ------

impl<'a, T: IntoElement<'a> + 'a> ApplyToElement<Button<'a>> for T {
    fn apply_to_element(self, button: &mut Button<'a>) {
        button.label = Some(Box::new(self.into_element()));
    }
} 

// ------ button::on_press(...)

pub struct OnPress(Box<dyn Fn()>);
pub fn on_press(on_press: impl FnOnce() + Clone + 'static) -> OnPress {
    OnPress(Box::new(move || on_press.clone()()))
}
impl<'a> ApplyToElement<Button<'a>> for OnPress {
    fn apply_to_element(self, button: &mut Button<'a>) {
        button.on_press = Some(self);
    }
}