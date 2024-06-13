#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_macros)]

mod canvas;
mod hanzi_lookup;
mod utils;

use wasm_bindgen::prelude::*;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().unwrap();

    let (_cm, div) = canvas::CanvasManager::new(128);
    body.append_child(&div).unwrap();

    Ok(())
}
