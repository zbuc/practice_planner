use std::fmt;

use wasm_bindgen::prelude::*;

use crate::components::tabs::TabDisplay;

#[wasm_bindgen(module = "/assets/js/tablature.js")]
extern "C" {
    pub fn create_tab();

    // pub type TablatureBindings;

    // #[wasm_bindgen(constructor)]
    // fn new() -> TablatureBindings;

    // #[wasm_bindgen(method, getter)]
    // fn number(this: &MyClass) -> u32;
    // #[wasm_bindgen(method, setter)]
    // fn set_number(this: &MyClass, number: u32) -> MyClass;
    // #[wasm_bindgen(method)]
    // pub fn render(this: &TablatureBindings); // -> String;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

// impl fmt::Display for TablatureBindings {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.render())
//     }
// }
