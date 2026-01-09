use log::info;
use wasm_bindgen::JsCast;

use crate::renderer::wgpu_state::WgpuState;
use crate::renderer::uniforms::Uniforms;

const SHADER_CODE: &str = include_str!("shaders/render.wgsl");

/// Renderer struct responsible for all rendering. Manages wgpu state and rendering pipeline(s).
pub struct Renderer<'window> {
    // wgpu state and resources
    wgpu_state: WgpuState<'window>,

    // render pipeline and bind groups/layouts
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

    // uniforms (viewport, cam_center, cam_half_size, num_bodies)
    uniforms: Uniforms,
    uniforms_buffer: wgpu::Buffer,

    // bodies buffer
    capacity_bodies: u32,
    bodies_buffer: wgpu::Buffer,
}

impl Renderer<'_> {
    /// Creates a new Renderer instance.
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
        wgpu_state.configure_surface(width, height);


        // create buffers
        let uniforms = Uniforms {
            view_port: [width, height],
            cam_center: [0.0, 0.0],
            cam_half_size: [10.0, 10.0],
            num_bodies: 0,
            _pad: 0,
        };

        // initialize bodies buffer
        let capacity_bodies = 1;
        let bodies_buffer = wgpu_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bodies buffer"),
            size: (capacity_bodies as u64) * std::mem::size_of::<[f32; 4]>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // initialize uniform buffer
        let uniforms_buffer = wgpu_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });


        // create render pipeline and bind group/layout
        let shader = wgpu_state.device.create_shader_module(wgpu::ShaderModuleDescriptor { 
            label: Some("render shader"), 
            source: wgpu::ShaderSource::Wgsl(SHADER_CODE.into())
        });

        let bind_group_layout = wgpu_state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = wgpu_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: bodies_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = wgpu_state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let swapchain_format = wgpu_state.surface.get_capabilities(&wgpu_state.adapter).formats[0];
        let render_pipeline = wgpu_state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
            label: Some("render pipeline"),
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

        Renderer {
            wgpu_state,
            render_pipeline,
            bind_group_layout,
            bind_group,
            uniforms,
            uniforms_buffer,
            capacity_bodies,
            bodies_buffer,
        }
    }

    fn update_uniforms_buffer(&self) {
        let uniforms_bytes = bytemuck::bytes_of(&self.uniforms);
        self.wgpu_state.queue.write_buffer(&self.uniforms_buffer, 0, uniforms_bytes);
    }

    fn recreate_bind_group(&mut self) {
        self.bind_group = self.wgpu_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.bodies_buffer.as_entire_binding(),
                },
            ],
        });
    }

    /// Renders a new frame to the canvas.
    pub fn render(&mut self) {
        // todo: temp, remove later
        self.fill_bodies_buffer(&vec![[0.0, 0.0, 0.5, 1.0], [3.0, 3.0, 0.5, 1.0]]);

        let frame = self.wgpu_state.surface
            .get_current_texture()
            .expect("failed to acquire next swap chain texture");

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.wgpu_state.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command encoder"),
            });
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
                label: Some("render pass"), 
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
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..6, 0..self.uniforms.num_bodies);
        }

        self.wgpu_state.queue.submit([encoder.finish()]);
        frame.present();
    }

    // todo: change [f32; 4] to Body struct when defined
    /// Fills the bodies buffer with given bodies data, so it can be rendered by the GPU.
    pub fn fill_bodies_buffer(&mut self, bodies: &Vec<[f32; 4]>) {
        self.uniforms.num_bodies = bodies.len() as u32;

        // resize buffer if needed
        if self.uniforms.num_bodies > self.capacity_bodies {
            self.capacity_bodies = (self.uniforms.num_bodies as f32 * 1.5).ceil() as u32;
            self.bodies_buffer = self.wgpu_state.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("bodies buffer"),
                size: (self.capacity_bodies as u64) * std::mem::size_of::<[f32; 4]>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.recreate_bind_group(); // bind group must be recreated to use new buffer
        }

        // upload data to buffer
        let data_bytes = bytemuck::cast_slice(&bodies[..]);
        self.wgpu_state.queue.write_buffer(&self.bodies_buffer, 0, data_bytes);

        // update uniform buffer with metadata (in case num_bodies changed)
        self.update_uniforms_buffer();
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

    /// Zooms the camera in or out, centered on the given pixel coordinates (in range [0, view_port.x/y]).
    pub fn zoom_camera(&mut self, px: f32, py: f32, zoom_factor: f32) {
        let cam_half_size = &mut self.uniforms.cam_half_size;
        let cam_center = &mut self.uniforms.cam_center;

        let ndc_x = (px / self.uniforms.view_port[0] as f32) * 2.0 - 1.0;
        let ndc_y = -((py / self.uniforms.view_port[1] as f32) * 2.0 - 1.0);

        let world_x = cam_center[0] + ndc_x * cam_half_size[0];
        let world_y = cam_center[1] + ndc_y * cam_half_size[1];

        cam_half_size[0] /= zoom_factor;
        cam_half_size[1] /= zoom_factor;

        cam_center[0] = world_x - ndc_x * cam_half_size[0];
        cam_center[1] = world_y - ndc_y * cam_half_size[1];
        self.update_uniforms_buffer();
    }

    /// Pans the camera by the given delta in pixels.
    pub fn pan_camera(&mut self, delta_px: f32, delta_py: f32) {
        let cam_half_size = &self.uniforms.cam_half_size;
        let delta_x = (delta_px / self.uniforms.view_port[0] as f32) * 2.0 * cam_half_size[0];
        let delta_y = -(delta_py / self.uniforms.view_port[1] as f32) * 2.0 * cam_half_size[1];
        self.uniforms.cam_center[0] += delta_x;
        self.uniforms.cam_center[1] += delta_y;
        self.update_uniforms_buffer();
    }
}