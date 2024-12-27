use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /*
    #[wasm_bindgen(js_name = btoa)]
    pub fn btoa(s: &str) -> String;
    */
    /*
    #[wasm_bindgen(js_name = alert)]
    pub fn alert(s: &str);
    */
    /*
    #[wasm_bindgen(js_name = confirm)]
    pub fn confirm(s: &str) -> bool;
    */
}

pub fn alert(s: &str) {
    leptos::prelude::window().alert_with_message(s).unwrap();
}

pub fn btoa(s: &str) -> String {
    leptos::prelude::window().btoa(s).unwrap()
}

pub fn confirm(s: &str) -> bool {
    leptos::prelude::window().confirm_with_message(s).unwrap()
}
