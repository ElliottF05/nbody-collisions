import type { Engine } from "../wasm/pkg/nbody_collisions";

export class InteractionHandler {
    private device: GPUDevice;
    private canvas: HTMLCanvasElement;
    private context: GPUCanvasContext;
    private canvasFormat: GPUTextureFormat;
    private engine: Engine;

    constructor(device: GPUDevice, canvas: HTMLCanvasElement, context: GPUCanvasContext, canvasFormat: GPUTextureFormat, engine: Engine) {
        this.device = device;
        this.canvas = canvas;
        this.context = context;
        this.canvasFormat = canvasFormat;
        this.engine = engine;

        this.resizeCanvasToDisplaySize();
        this.addResizeListener();
    }

    private addResizeListener() {
        window.addEventListener('resize', () => {
            this.resizeCanvasToDisplaySize();
        });
    }

    private resizeCanvasToDisplaySize() {
        const dpr = window.devicePixelRatio || 1;

        const rect = this.canvas.getBoundingClientRect();
        const width = rect.width * dpr;
        const height = rect.height * dpr;

        if (this.canvas.width !== width || this.canvas.height !== height) {
            this.canvas.width = width;
            this.canvas.height = height;

            this.context.configure({
                device: this.device,
                format: this.canvasFormat,
                alphaMode: "premultiplied",
            });

            // notify the engine about the resize
            this.engine.resize(width, height);
        }
    }
}