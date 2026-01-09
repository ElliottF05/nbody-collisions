use wasm_bindgen::JsCast;

pub struct WgpuState<'window> {
    pub surface: wgpu::Surface<'window>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl WgpuState<'_> {
    pub async fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

        let surface_target = wgpu::SurfaceTarget::Canvas(canvas);
        let surface = instance
            .create_surface(surface_target)
            .expect("failed to create surface from canvas");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("failed to find an appropriate adapter");
            
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("failed to create device");

        WgpuState {
            surface,
            adapter,
            device,
            queue,
        }
    }

    pub fn configure_surface(&self, width: u32, height: u32) {
        let config = self.surface
            .get_default_config(&self.adapter, width, height)
            .expect("failed to get default config");
        self.surface.configure(&self.device, &config);
    }
}