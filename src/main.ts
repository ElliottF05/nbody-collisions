import { Engine } from '../wasm/pkg/nbody_collisions';
import { initDeviceAndContext } from './gpuSetup';
import { InteractionHandler } from './interaction';

const FPS = 60;
const SUBSTEPS = 5;
const DELTA_TIME = 1 / (FPS * SUBSTEPS);

async function main() {
    const { device, canvas, context, canvasFormat } = await initDeviceAndContext('canvas');

    const engine = await Engine.create();
    const interactionHandler = new InteractionHandler(device, canvas, context, canvasFormat, engine);

    function frame() {
        for (let i = 0; i < SUBSTEPS; i++) {
            engine.update(DELTA_TIME);
        }
        engine.render();
        requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
}
main();