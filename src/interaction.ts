import type { Engine } from "../wasm/pkg/nbody_collisions";

export class InteractionHandler {
    private device: GPUDevice;
    private canvas: HTMLCanvasElement;
    private context: GPUCanvasContext;
    private canvasFormat: GPUTextureFormat;
    private engine: Engine;

    // interaction state
    private isMouseDown: boolean;
    private lastMouseCanvasPos: [number, number];


    constructor(device: GPUDevice, canvas: HTMLCanvasElement, context: GPUCanvasContext, canvasFormat: GPUTextureFormat, engine: Engine) {
        this.device = device;
        this.canvas = canvas;
        this.context = context;
        this.canvasFormat = canvasFormat;
        this.engine = engine;

        // interaction state
        this.isMouseDown = false;
        this.lastMouseCanvasPos = [0, 0];

        this.resizeCanvasToDisplaySize();
        
        this.addResizeListener();
        this.addZoomListener();
        this.addPanListeners();
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

    private clientToCanvasCoords(clientX: number, clientY: number): [number, number] {
        const rect = this.canvas.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        const canvasX = (clientX - rect.left) * dpr;
        const canvasY = (clientY - rect.top) * dpr;
        return [canvasX, canvasY];
    }

    private addZoomListener() {
        this.canvas.addEventListener('wheel', (event) => {
            event.preventDefault();

            const zoomSpeed = 0.0015;
            const zoomFactor = Math.exp(-event.deltaY * zoomSpeed);

            const [px, py] = this.clientToCanvasCoords(event.clientX, event.clientY);

            // notify the engine about the zoom
            this.engine.zoom_camera(px, py, zoomFactor);
        }, { passive: false });
    }

    private addPanListeners() {
        this.canvas.addEventListener("pointerdown", (e) => {
            if (e.button !== 0) {
                return; // must be left mouse button
            }
            this.isMouseDown = true;
            this.lastMouseCanvasPos = this.clientToCanvasCoords(e.clientX, e.clientY);
            this.canvas.setPointerCapture(e.pointerId);
        });

        this.canvas.addEventListener("pointermove", (e) => {
            if (!this.isMouseDown) {
                return;
            }
            const [canvasX, canvasY] = this.clientToCanvasCoords(e.clientX, e.clientY);
            const deltaX = canvasX - this.lastMouseCanvasPos[0];
            const deltaY = canvasY - this.lastMouseCanvasPos[1];
            this.lastMouseCanvasPos = [canvasX, canvasY];

            this.engine.pan_camera(-deltaX, -deltaY);
        });

        const endPan = (e: PointerEvent) => {
            if (!this.isMouseDown) {
                return;
            }
            this.isMouseDown = false;
            try {
                this.canvas.releasePointerCapture(e.pointerId);
            } catch {
                // do nothing
            }
        }

        this.canvas.addEventListener("pointerup", endPan);
        this.canvas.addEventListener("pointercancel", endPan);
    }
}