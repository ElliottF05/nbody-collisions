use wasm_bindgen::prelude::*;

// wasm-pack build --features console_error_panic_hook (feature is optional)

mod engine;
mod simulation;
mod renderer;

#[wasm_bindgen(start)]
pub fn start() {
    
    // set up console error panic hook for better error messages in wasm
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // initialize logger (use debug! macro for logging)
    console_log::init_with_level(log::Level::Debug).expect("could not initialize logger");
}