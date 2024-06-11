use core::fmt;
use std::fmt::Formatter;
use std::sync::Arc;

use crate::utils::{self, WINDOW_HEIGHT, WINDOW_WIDTH};
use image::GenericImage;
use log::error;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupLayout, CommandEncoder, Device, PipelineCompilationOptions, RenderPipeline,
    ShaderModule, TextureFormat, VertexBufferLayout,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct State {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,
    display_render_pipeline: wgpu::RenderPipeline,
    canvas_texture: wgpu::Texture,
    canvas_render_pipeline: wgpu::RenderPipeline,
    canvas_bind_group: wgpu::BindGroup,
    offset_bind_group: wgpu::BindGroup,
    drawing_offset_buffer: wgpu::Buffer,
}

pub type RenderCommands = CommandEncoder;
impl State {
    const CLEAR_COLOR: wgpu::Color = wgpu::Color {
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    };
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Arc<Window>) -> Self {
        let size = PhysicalSize {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        };

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            dx12_shader_compiler: Default::default(),
            flags: Default::default(),
            gles_minor_version: Default::default(),
        });
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = {
            match instance.create_surface(window.clone()) {
                Ok(surface) => surface,
                Err(err) => {
                    error!("Failed to create surface: {:?}", err);
                    unreachable!()
                }
            }
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
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
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        let render_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/render_shader.wgsl"));
        let canvas_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/canvas_shader.wgsl"));
        //The canvas texture
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: utils::WINDOW_WIDTH,
                height: utils::WINDOW_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            label: None,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        };
        let texture = device.create_texture(&texture_desc);
        let canvas_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let canvas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        let canvas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&canvas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&canvas_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        let offset_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Offset Buffer"),
            contents: bytemuck::cast_slice(&[0.0f32, 0.0f32, 0.0f32, 0.0f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let offset_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Offset Buffer Layout"),
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
            });
        let offset_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Offset Bind Group"),
            layout: &offset_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: offset_buffer.as_entire_binding(),
            }],
        });
        let render_pipeline: RenderPipeline = Self::create_pipeline(
            &device,
            config.format,
            render_shader,
            &[&texture_bind_group_layout],
            &[],
        );
        let canvas_render_pipeline: RenderPipeline = Self::create_pipeline(
            &device,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            canvas_shader,
            &[&offset_bind_group_layout],
            &[Vertex::desc()],
        );
        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            display_render_pipeline: render_pipeline,
            canvas_texture: texture,
            canvas_render_pipeline,
            canvas_bind_group,
            offset_bind_group,
            drawing_offset_buffer: offset_buffer,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
    pub fn begin_render(&mut self) -> RenderCommands {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            })
    }
    pub fn end_render(&mut self, commands: RenderCommands) {
        self.queue.submit(std::iter::once(commands.finish()));
    }

    pub fn clear_screen(&mut self, commands: &mut RenderCommands) {
        let color_attachment_operation = wgpu::Operations {
            load: wgpu::LoadOp::Clear(Self::CLEAR_COLOR),
            store: wgpu::StoreOp::Store,
        };
        {
            let view = self
                .canvas_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_pass = commands.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Canvas Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: color_attachment_operation,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.canvas_render_pipeline);
        }
    }

    pub fn draw_buffer(
        &mut self,
        commands: &mut RenderCommands,
        buffer: &wgpu::Buffer,
        offset: [f32; 2],
    ) {
        let color_attachment_operation = wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
        };
        {
            self.queue.write_buffer(
                &self.drawing_offset_buffer,
                0,
                bytemuck::cast_slice(&offset),
            );
            let view = self
                .canvas_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_pass = commands.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Draw Buffer Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: color_attachment_operation,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.canvas_render_pipeline);
            render_pass.set_vertex_buffer(0, buffer.slice(..));
            render_pass.set_bind_group(0, &self.offset_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
    }
    //Creates a buffer for this state, which can be used to draw to the screen
    pub fn make_test_buffer(&mut self, vertices: &[Vertex]) -> wgpu::Buffer {
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        vertex_buffer
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
        let color_attachment_operation = wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }),
            store: wgpu::StoreOp::Store,
        };
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: color_attachment_operation,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.display_render_pipeline);
            render_pass.set_bind_group(0, &self.canvas_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
    //The buffer for the copy operation must have the width a multiple of 256
    fn get_necessary_buffer_width(width: u32) -> u32 {
        //TODO: replace chatgpt code here
        let mut necessary_width = width;
        while necessary_width % 256 != 0 {
            necessary_width += 1;
        }
        necessary_width
    }
    //EXTRACTS THE FRAMEBUFFER FROM THE GPU, THE FORMAT IS NOT DEFINED YET(considered BGRA8Srgb for now)
    //Since this is intended to only be called at the end of the program(and only once) it should be fine to allocate the buffer here
    pub async fn extract_framebuffer(&self) -> image::RgbaImage {
        let u32_size = std::mem::size_of::<u32>() as u32;
        let output_buffer_size = (u32_size
            * utils::WINDOW_HEIGHT
            * Self::get_necessary_buffer_width(utils::WINDOW_WIDTH))
            as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
                // this tells wpgu that we want to read this buffer from the cpu
                | wgpu::BufferUsages::MAP_READ,
            label: Some("Buffer containing the canvas for sending to the form"),
            mapped_at_creation: false,
        };
        let output_buffer = self.device.create_buffer(&output_buffer_desc);
        let output = &self.canvas_texture;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &output,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        Self::get_necessary_buffer_width(utils::WINDOW_WIDTH) * u32_size,
                    ),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: utils::WINDOW_WIDTH,
                height: utils::WINDOW_HEIGHT,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));
        {
            // need 2 pipelines as we can't use the surface render pipeline because of the color attachment
            let buffer_slice = output_buffer.slice(..);
            // the future. Otherwise the application will freeze.
            let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            self.device.poll(wgpu::Maintain::Wait);
            rx.receive().await.unwrap().unwrap();

            let data = buffer_slice.get_mapped_range();
            image::RgbaImage::from_raw(
                Self::get_necessary_buffer_width(utils::WINDOW_WIDTH),
                utils::WINDOW_HEIGHT,
                data.to_vec(),
            )
            .unwrap()
            .sub_image(0, 0, utils::WINDOW_WIDTH, utils::WINDOW_HEIGHT)
            .to_image()
        }
    }

    fn create_pipeline(
        device: &Device,
        format: TextureFormat,
        shader: ShaderModule,
        bind_group_layouts: &[&BindGroupLayout],
        buffers: &[VertexBufferLayout],
    ) -> RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: bind_group_layouts,
                push_constant_ranges: &[],
            });
        let screen_pipeline_fragment_target = [Some(wgpu::ColorTargetState {
            format: format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let screen_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers,
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &screen_pipeline_fragment_target,
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        };
        device.create_render_pipeline(&screen_pipeline_descriptor)
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("size", &self.size)
            .field("window", &self.window)
            .finish()
    }
}
