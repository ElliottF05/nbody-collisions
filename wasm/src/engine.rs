use wasm_bindgen::prelude::*;

use crate::simulation::Simulation;
use crate::renderer::Renderer;

#[wasm_bindgen]
struct Engine {
    simulation: Simulation,
    renderer: Renderer<'static>,
}

// engine functions exposed to javascript
#[wasm_bindgen]
impl Engine {

    #[allow(unused)] // compiler thinks "static_method_of" is a (unused) variable for some reason
    #[wasm_bindgen(static_method_of = Engine)]
    pub async fn create() -> Engine {
        let simulation = Simulation::new();
        let renderer = Renderer::new().await;
        Engine { simulation, renderer }
    }

    pub fn update(&mut self, dt: f32) {
        self.simulation.update(dt);
    }

    pub fn render(&mut self) {
        self.renderer.render();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    pub fn zoom_camera(&mut self, px: f32, py: f32, zoom_factor: f32) {
        self.renderer.zoom_camera(px, py, zoom_factor);
    }
}

// internal engine functions
impl Engine {
    // todo
}