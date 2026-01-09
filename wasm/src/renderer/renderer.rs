use wasm_bindgen::JsCast;

use crate::renderer::wgpu_state::WgpuState;
use crate::renderer::uniforms::Uniforms;

const SHADER_CODE: &str = include_str!("shaders/render.wgsl");

/// Renderer struct responsible for all rendering. Manages wgpu state and rendering pipeline(s).
pub struct Renderer<'window> {
    // wgpu state and resources
    wgpu_state: WgpuState<'window>,
    render_pipeline: wgpu::RenderPipeline,

    // uniforms (viewport, cam_center, cam_half_size, num_bodies)
    uniforms: Uniforms,

    // bodies buffer
    capacity_bodies: u32,
    bodies_buffer: wgpu::Buffer,

    // metadata uniforms
    uniforms_buffer: wgpu::Buffer,
}

impl Renderer<'_> {
    pub async fn new() -> Self {
        // create wgpu state first (requires canvas element)
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("should have a document on window");
        let canvas = document
            .get_element_by_id("canvas")
            .expect("document should have a canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element should be a canvas");

        let (width, height) = (canvas.width(), canvas.height());
        let wgpu_state = WgpuState::new(canvas.clone()).await;

        // create render pipeline 
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

        wgpu_state.configure_surface(width, height);

        let uniforms = Uniforms {
            view_port: [width, height],
            cam_center: [0.0, 0.0],
            cam_half_size: [10.0, 10.0],
            num_bodies: 0,
            _pad: 0,
        };

        // initialize bodies buffer
        let capacity_bodies = 0;
        let bodies_buffer = wgpu_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 0, 
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // initialize uniform buffer
        let uniforms_buffer = wgpu_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Renderer {
            wgpu_state,
            render_pipeline,
            uniforms,
            capacity_bodies,
            bodies_buffer,
            uniforms_buffer,
        }
    }

    // todo: change [f32; 4] to Body struct when defined
    /// Fills the bodies buffer with given bodies data, so it can be rendered by the GPU.
    pub fn fill_bodies_buffer(&mut self, bodies: &Vec<[f32; 4]>) {
        self.uniforms.num_bodies = bodies.len() as u32;

        // resize buffer if needed
        if self.uniforms.num_bodies > self.capacity_bodies {
            self.capacity_bodies = (self.uniforms.num_bodies as f32 * 1.5).ceil() as u32;
            self.bodies_buffer = self.wgpu_state.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (self.capacity_bodies as u64) * std::mem::size_of::<[f32; 4]>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // upload data to buffer
        let data_bytes = bytemuck::cast_slice(&bodies[..]);
        self.wgpu_state.queue.write_buffer(&self.bodies_buffer, 0, data_bytes);

        // update uniform buffer with metadata (in case num_bodies changed)
        self.update_uniforms_buffer();
    }

    fn update_uniforms_buffer(&self) {
        let uniforms_bytes = bytemuck::bytes_of(&self.uniforms);
        self.wgpu_state.queue.write_buffer(&self.uniforms_buffer, 0, uniforms_bytes);
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

    /// Resizes the renderer to the given width and height (in pixels) of the viewport.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.uniforms.view_port = [width, height];
        let aspect = width as f32 / height as f32;
        self.uniforms.cam_half_size[0] = self.uniforms.cam_half_size[1] * aspect;

        // reconfigure surface and update uniforms buffer
        self.wgpu_state.configure_surface(width, height);
        self.update_uniforms_buffer();
    }
}