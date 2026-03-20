use std::{cell::Cell, rc::Rc};

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window not available"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document not available"))?;

    let root = document
        .get_element_by_id("app")
        .ok_or_else(|| JsValue::from_str("#app not found"))?;

    let container = document.create_element("div")?;
    container.set_attribute("id", "counter-app")?;

    let title = document.create_element("h1")?;
    title.set_text_content(Some("Rust DOM Counter"));

    let value = document.create_element("p")?;
    value.set_attribute("id", "count-value")?;
    value.set_text_content(Some("Count: 0"));

    let button = document.create_element("button")?;
    button.set_attribute("id", "increment")?;
    button.set_text_content(Some("Increment"));

    container.append_child(&title)?;
    container.append_child(&value)?;
    container.append_child(&button)?;
    root.append_child(&container)?;

    let count = Rc::new(Cell::new(0_i32));
    let value_for_click = value.clone();
    let count_for_click = Rc::clone(&count);

    let on_click = Closure::wrap(Box::new(move || {
        let next = count_for_click.get() + 1;
        count_for_click.set(next);
        value_for_click.set_text_content(Some(&format!("Count: {next}")));
    }) as Box<dyn FnMut()>);

    button.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
    on_click.forget();

    Ok(())
}
