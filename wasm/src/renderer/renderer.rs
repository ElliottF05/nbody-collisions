use wasm_bindgen::JsCast;

const SHADER_CODE: &str = include_str!("shaders/render.wgsl");

pub struct Renderer<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
}

impl Renderer<'_> {
    pub async fn new() -> Self {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("should have a document on window");
        let canvas = document
            .get_element_by_id("canvas")
            .expect("document should have a canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element should be a canvas");

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

        let (width, height) = (canvas.width(), canvas.height());
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
            
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(SHADER_CODE.into())
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let swapchain_format = surface.get_capabilities(&adapter).formats[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
            label: None, 
            layout: Some(&pipeline_layout), 
            vertex: wgpu::VertexState {
                module: &shader, 
                entry_point: Some("vertex_main"), 
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }, 
            fragment: Some(wgpu::FragmentState {
                module: &shader, 
                entry_point: Some("fragment_main"), 
                compilation_options: wgpu::PipelineCompilationOptions::default(), 
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(), 
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),  
            multiview_mask: None, 
            cache: None, 
        });

        let config = surface
            .get_default_config(&adapter, width, height)
            .expect("failed to get default config");
        surface.configure(&device, &config);

        Renderer {
            surface,
            device,
            queue,
            render_pipeline,
        }
    }

    /// Renders a new frame to the canvas.
    pub fn render(&self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swap chain texture");

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: None,
            });
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
                label: None, 
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                    view: &view, 
                    depth_slice: None,
                    resolve_target: None, 
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), 
                        store: wgpu::StoreOp::Store, 
                    }, 
                })],
                depth_stencil_attachment: None, 
                timestamp_writes: None, 
                occlusion_query_set: None, 
                multiview_mask: None, 
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        frame.present();
    }
}