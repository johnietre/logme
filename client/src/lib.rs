use leptos::prelude::*;
use wasm_bindgen::prelude::*;

mod app;
use app::*;

mod console;

mod datetime_input;

mod utils;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    init_wasm_hooks();
    mount_to_body(|| view! { <App /> });
    Ok(())
}

fn init_wasm_hooks() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
}
