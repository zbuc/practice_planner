use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/assets/js/tablature.js")]
extern "C" {
    pub fn create_tab();
}
