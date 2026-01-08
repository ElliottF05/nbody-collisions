use wasm_bindgen::prelude::*;

use crate::simulation::Simulation;
use crate::renderer::Renderer;
use crate::utils::set_panic_hook;

#[wasm_bindgen]
struct Engine {
    simulation: Simulation,
    renderer: Renderer<'static>,
}

// engine functions exposed to javascript
#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(static_method_of = Engine)]
    pub async fn create() -> Engine {
        set_panic_hook(); // set panic hook (if enabled) for better wasm error messages
        let simulation = Simulation::new();
        let renderer = Renderer::new().await;
        Engine { simulation, renderer }
    }

    pub fn update(&mut self, dt: f32) {
        self.simulation.update(dt);
    }

    pub fn render(&self) {
        self.renderer.render();
    }
}

// internal engine functions
impl Engine {
    // todo
}