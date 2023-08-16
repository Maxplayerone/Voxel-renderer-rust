use std::iter;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::camera::{Camera, Projection};
use crate::chunk;
use crate::depth_texture;

pub struct Render {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    window: winit::window::Window,

    pipeline: wgpu::RenderPipeline,

    camera_buffer: wgpu::Buffer,
    camera_uniform: CameraUniform,
    camera_bind_group: wgpu::BindGroup,

    chunks: Vec<chunk::ChunkMeshData>,
    depth_texture: depth_texture::DepthTexture,
}

enum RenderingMode {
    Fill,
    Wireframe,
}

enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

fn get_chunk(index: usize, width: usize, dir: Direction) -> Option<usize> {
    match dir {
        Direction::Left => {
            let (x, z) = index_to_coords(index, width);
            if x < 16 {
                return None;
            }
            let index_left_chunk = coords_to_index(x - 16, z, width);
            return Some(index_left_chunk);
        }
        _ => return None,
    }
}

fn coords_to_index(x: usize, z: usize, width: usize) -> usize {
    (z / 16 * width + x / 16).try_into().unwrap()
}

fn index_to_coords(index: usize, width: usize) -> (usize, usize) {
    let x = (index % width as usize) * 16;
    let z = (index / width as usize) * 16;
    (x, z)
}

impl Render {
    pub async fn new(window: winit::window::Window) -> Self {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::POLYGON_MODE_LINE,
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let camera_uniform = CameraUniform::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        };

        let depth_texture = depth_texture::DepthTexture::create_depth_texture(&device, &config, "depth_texture");

        let pipeline = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            config.format,
            &[chunk::Vertex::desc()],
            shader,
            RenderingMode::Wireframe,
        );


        let mut chunks = Vec::new();
        let width = 4;
        for z in 0..1 {
            for x in 0..width {
                let mut chunk =
                    chunk::ChunkMeshData::new(cgmath::Vector3::<usize>::new(x * 16, 0, z * 16));
                chunk.generate_data();
                chunks.push(chunk);
            }
        }
        let mut total_faces: u32 = 0;
        for i in 0..chunks.len() {
            if let Some(left_chunk_index) = get_chunk(i, width, Direction::Left) {
                let left_chunk_data = chunks[left_chunk_index].chunk_data.clone();
                total_faces += chunks[i].generate_mesh(&left_chunk_data);
            }
            //chunk.generate_mesh(left_chunk_data, right_chunk_data, top_chunk_data, bottom_chunk_data);
        }
        println!("total faces {}", total_faces);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,

            pipeline,
            chunks,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            depth_texture,
        }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn width(&self) -> f32 {
        self.config.width as f32
    }

    pub fn height(&self) -> f32 {
        self.config.height as f32
    }

    //I prefer to have a method for that then to have a random
    //one public thing in the struct
    pub fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) -> bool {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = depth_texture::DepthTexture::create_depth_texture(&self.device, &self.config, "depth texture");
            return true;
        }
        false
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let _is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Key1 => {
                        let camera_bind_group_layout = self.device.create_bind_group_layout(
                            &wgpu::BindGroupLayoutDescriptor {
                                entries: &[wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    visibility: wgpu::ShaderStages::VERTEX,
                                    ty: wgpu::BindingType::Buffer {
                                        ty: wgpu::BufferBindingType::Uniform,
                                        has_dynamic_offset: false,
                                        min_binding_size: None,
                                    },
                                    count: None,
                                }],
                                label: Some("camera_bind_group_layout"),
                            },
                        );
                        let render_pipeline_layout =
                            self.device
                                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                                    label: Some("Render Pipeline Layout"),
                                    bind_group_layouts: &[&camera_bind_group_layout],
                                    push_constant_ranges: &[],
                                });
                        let shader = wgpu::ShaderModuleDescriptor {
                            label: Some("Shader"),
                            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                        };
                        self.pipeline = create_render_pipeline(
                            &self.device,
                            &render_pipeline_layout,
                            self.config.format,
                            &[chunk::Vertex::desc()],
                            shader,
                            RenderingMode::Fill,
                        );
                        true
                    }
                    VirtualKeyCode::Key2 => {
                        let camera_bind_group_layout = self.device.create_bind_group_layout(
                            &wgpu::BindGroupLayoutDescriptor {
                                entries: &[wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    visibility: wgpu::ShaderStages::VERTEX,
                                    ty: wgpu::BindingType::Buffer {
                                        ty: wgpu::BufferBindingType::Uniform,
                                        has_dynamic_offset: false,
                                        min_binding_size: None,
                                    },
                                    count: None,
                                }],
                                label: Some("camera_bind_group_layout"),
                            },
                        );
                        let render_pipeline_layout =
                            self.device
                                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                                    label: Some("Render Pipeline Layout"),
                                    bind_group_layouts: &[&camera_bind_group_layout],
                                    push_constant_ranges: &[],
                                });
                        let shader = wgpu::ShaderModuleDescriptor {
                            label: Some("Shader"),
                            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                        };
                        self.pipeline = create_render_pipeline(
                            &self.device,
                            &render_pipeline_layout,
                            self.config.format,
                            &[chunk::Vertex::desc()],
                            shader,
                            RenderingMode::Wireframe,
                        );
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update(&mut self, camera: &Camera, projection: &Projection) {
        self.camera_uniform.update_view_proj(camera, projection);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let mut vertex_buffers = Vec::new();
        let mut index_buffers = Vec::new();
        let mut indiceses = Vec::new();
        for chunk in self.chunks.iter_mut() {
            let (vertex_buffer, index_buffer, indices) = chunk.build(&self.device);
            vertex_buffers.push(vertex_buffer);
            index_buffers.push(index_buffer);
            indiceses.push(indices);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment{
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations{
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipeline);
            for i in 0..self.chunks.len() {
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffers[i].slice(..));
                render_pass.set_index_buffer(index_buffers[i].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..indiceses[i], 0, 0..1);
            }
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
    mode: RenderingMode,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: match mode {
                RenderingMode::Fill => wgpu::PolygonMode::Fill,
                RenderingMode::Wireframe => wgpu::PolygonMode::Line,
            },
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState{
            format: depth_texture::DepthTexture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
    })
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        let view_proj = cgmath::Matrix4::identity();
        Self {
            view_proj: view_proj.into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_proj =
            (OPENGL_TO_WGPU_MATRIX * projection.get_projection() * camera.get_view()).into();
    }
}
