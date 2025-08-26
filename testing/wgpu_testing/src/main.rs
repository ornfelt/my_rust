use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    offset: [f32; 2],
    _pad: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}
impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as u64,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { pos: [ 0.0,  0.6], color: [1.0, 0.2, 0.2] },
    Vertex { pos: [-0.6, -0.6], color: [0.2, 1.0, 0.2] },
    Vertex { pos: [ 0.6, -0.6], color: [0.2, 0.2, 1.0] },
];
const INDICES: &[u16] = &[0, 1, 2];

const SHADER: &str = r#"
struct Uniforms { offset: vec2<f32>, _pad: vec2<f32> };
@group(0) @binding(0) var<uniform> ubo: Uniforms;

struct VSOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@location(0) in_pos: vec2<f32>, @location(1) in_color: vec3<f32>) -> VSOut {
  var out: VSOut;
  out.pos = vec4<f32>(in_pos + ubo.offset, 0.0, 1.0);
  out.color = in_color;
  return out;
}

@fragment
fn fs_main(@location(0) color: vec3<f32>) -> @location(0) vec4<f32> {
  return vec4<f32>(color, 1.0);
}
"#;

struct Gpu<'w> {
    surface: wgpu::Surface<'w>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    num_indices: u32,
    uniform: Uniforms,
    uniform_buf: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl<'w> Gpu<'w> {
    fn new(window: &'w winit::window::Window) -> Self {
        let size = window.inner_size();

        // Force some backend
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            //backends: wgpu::Backends::VULKAN,
            //backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).expect("create surface");

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("No adapter");

        // Print backend + adapter info
        let info = adapter.get_info();
        println!(
            "Using backend: {:?}, name: {}, driver: {}",
            info.backend, info.name, info.driver
        );

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))
        .expect("device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_caps.present_modes[0],
            // or:
            //present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Buffers
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Uniforms
        let uniform = Uniforms { offset: [0.0, 0.0], _pad: [0.0, 0.0] };
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        // Shader & pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buf,
            index_buf,
            num_indices: INDICES.len() as u32,
            uniform,
            uniform_buf,
            uniform_bind_group,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.size = size;
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update_uniform(&mut self) {
        self.queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&self.uniform));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder"),
        });

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.06, g: 0.06, b: 0.08, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            rp.set_pipeline(&self.render_pipeline);
            rp.set_bind_group(0, &self.uniform_bind_group, &[]);
            rp.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rp.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rp.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

fn main() {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("DirectX 12 (wgpu) Triangle - WASD moves")
        .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    // Leak the window to obtain a 'static reference
    let window: &'static winit::window::Window = Box::leak(Box::new(window));

    // state
    let mut pressed_w = false;
    let mut pressed_a = false;
    let mut pressed_s = false;
    let mut pressed_d = false;

    let speed: f32 = 0.8;
    let mut last = std::time::Instant::now();

    // Now the Option can hold Gpu<'static>
    let mut maybe_gpu: Option<Gpu<'static>> = None;

    event_loop
        .run(move |event, elwt| {
            match event {
                Event::Resumed => {
                    if maybe_gpu.is_none() {
                        maybe_gpu = Some(Gpu::new(window));
                    }
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(sz) => {
                        if let Some(g) = maybe_gpu.as_mut() { g.resize(sz); }
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        let down = event.state == ElementState::Pressed;
                        use winit::keyboard::{KeyCode::*, PhysicalKey::Code};
                        match event.physical_key {
                            Code(KeyW) => pressed_w = down,
                            Code(KeyA) => pressed_a = down,
                            Code(KeyS) => pressed_s = down,
                            Code(KeyD) => pressed_d = down,
                            Code(Escape) if down => elwt.exit(),
                            _ => {}
                        }
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    if let Some(gpu) = maybe_gpu.as_mut() {
                        let now = std::time::Instant::now();
                        let dt = (now - last).as_secs_f32();
                        last = now;

                        let mut dx = 0.0f32;
                        let mut dy = 0.0f32;
                        if pressed_w { dy += speed * dt; }
                        if pressed_s { dy -= speed * dt; }
                        if pressed_a { dx -= speed * dt; }
                        if pressed_d { dx += speed * dt; }

                        gpu.uniform.offset[0] = (gpu.uniform.offset[0] + dx).clamp(-1.2, 1.2);
                        gpu.uniform.offset[1] = (gpu.uniform.offset[1] + dy).clamp(-1.2, 1.2);
                        gpu.update_uniform();

                        match gpu.render() {
                            Ok(()) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => gpu.resize(gpu.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => { eprintln!("Out of memory"); elwt.exit(); }
                            Err(e) => eprintln!("render error: {e:?}"),
                        }
                    }

                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}
