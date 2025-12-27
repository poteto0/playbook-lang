use wasm_bindgen::prelude::*;

pub mod ast;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod renderer;

pub use renderer::Renderer;

#[wasm_bindgen]
pub fn render_playbook(input: &str) -> String {
    let renderer = Renderer::new();
    renderer.render(input)
}
