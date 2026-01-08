import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";
// import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [
    wasm(),

    // topLevelAwait() // not working for now with vite rolldown, 
    // this plugin is only needed for older browsers?
  ]
});