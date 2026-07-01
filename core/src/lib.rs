use wasm_bindgen::prelude::*;

pub mod ast;
pub mod constants;
mod geometry;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod renderer;

pub use renderer::Renderer;

#[wasm_bindgen]
pub fn render_playbook(input: &str) -> Result<String, JsValue> {
    let renderer = Renderer::new();
    renderer.render(input).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn play_playbook(input: &str) -> Result<String, JsValue> {
    let renderer = Renderer::new();
    renderer.play(input).map_err(|e| JsValue::from_str(&e))
}
