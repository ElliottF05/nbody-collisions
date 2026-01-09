use wasm_bindgen::JsCast;

use crate::renderer::wgpu_state::WgpuState;

const SHADER_CODE: &str = include_str!("shaders/render.wgsl");

pub struct Renderer<'window> {
    // wgpu state and resources
    wgpu_state: WgpuState<'window>,
    render_pipeline: wgpu::RenderPipeline,

    // camera state
    view_port: (u32, u32),
    cam_center: (f32, f32),
    cam_half_size: (f32, f32),

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

        let (width, height) = (canvas.width(), canvas.height());
        let wgpu_state = WgpuState::new(canvas.clone()).await;
            
        let shader = wgpu_state.device.create_shader_module(wgpu::ShaderModuleDescriptor { 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(SHADER_CODE.into())
        });

        let pipeline_layout = wgpu_state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let swapchain_format = wgpu_state.surface.get_capabilities(&wgpu_state.adapter).formats[0];

        let render_pipeline = wgpu_state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
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

        let config = wgpu_state.surface
            .get_default_config(&wgpu_state.adapter, width, height)
            .expect("failed to get default config");
        wgpu_state.surface.configure(&wgpu_state.device, &config);

        Renderer {
            wgpu_state,
            render_pipeline,
            view_port: (width, height),
            cam_center: (0.0, 0.0),
            cam_half_size: (10.0, 10.0),
        }
    }

    /// Renders a new frame to the canvas.
    pub fn render(&self) {
        let frame = self.wgpu_state.surface
            .get_current_texture()
            .expect("failed to acquire next swap chain texture");

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.wgpu_state.device
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

        self.wgpu_state.queue.submit([encoder.finish()]);
        frame.present();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.view_port = (width, height);
        let aspect = width as f32 / height as f32;
        self.cam_half_size = (self.cam_half_size.1 * aspect, self.cam_half_size.1);

        self.wgpu_state.configure_surface(width, height);
    }
}