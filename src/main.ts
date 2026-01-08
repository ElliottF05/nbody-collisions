import { Engine } from '../wasm/pkg/nbody_collisions';

const FPS = 60;
const SUBSTEPS = 5;
const DT = 1 / (FPS * SUBSTEPS);

async function main() {
    const engine = await Engine.create();

    function frame() {
        for (let i = 0; i < SUBSTEPS; i++) {
            engine.update(DT);
        }
        engine.render();
        requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
}

main();