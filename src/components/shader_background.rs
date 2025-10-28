use dioxus::prelude::*;
use dioxus_native::{CustomPaintCtx, CustomPaintSource, DeviceHandle, TextureHandle, use_wgpu};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;
use wgpu::*;

#[allow(dead_code)] // Shader background system - complete implementation pending UI integration
pub enum ShaderMessage {
    UpdateTime(f32),
}

#[allow(dead_code)] // Shader background system - complete implementation pending UI integration
enum ShaderState {
    Active(Box<ActiveShaderRenderer>),
    Suspended,
}

#[allow(dead_code)] // Shader background system - complete implementation pending UI integration
#[derive(Clone)]
struct TextureAndHandle {
    texture: Texture,
    handle: TextureHandle,
}

#[allow(dead_code)] // Shader background system - complete implementation pending UI integration
struct ActiveShaderRenderer {
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    displayed_texture: Option<TextureAndHandle>,
    next_texture: Option<TextureAndHandle>,
    time_buffer: Buffer,
    time_bind_group: BindGroup,
}

#[allow(dead_code)] // Shader background system - complete implementation pending UI integration
pub struct ShaderPaintSource {
    state: ShaderState,
    start_time: Instant,
    tx: Sender<ShaderMessage>,
    rx: Receiver<ShaderMessage>,
}

#[allow(dead_code)] // Shader background system - complete implementation pending UI integration
impl ShaderPaintSource {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            state: ShaderState::Suspended,
            start_time: Instant::now(),
            tx,
            rx,
        }
    }

    pub fn sender(&self) -> Sender<ShaderMessage> {
        self.tx.clone()
    }

    fn process_messages(&mut self) {
        loop {
            match self.rx.try_recv() {
                Err(_) => return,
                Ok(msg) => match msg {
                    ShaderMessage::UpdateTime(_time) => {
                        // Time updates trigger re-render for animation
                        // Actual time is calculated from start_time in render
                    }
                },
            }
        }
    }

    fn render_shader(
        &mut self,
        ctx: CustomPaintCtx<'_>,
        width: u32,
        height: u32,
    ) -> Option<TextureHandle> {
        if width == 0 || height == 0 {
            return None;
        }
        let ShaderState::Active(state) = &mut self.state else {
            return None;
        };

        state.render(ctx, width, height, &self.start_time)
    }
}

impl CustomPaintSource for ShaderPaintSource {
    fn resume(&mut self, device_handle: &DeviceHandle) {
        let device = &device_handle.device;
        let queue = &device_handle.queue;
        let active_state = ActiveShaderRenderer::new(device, queue);
        self.state = ShaderState::Active(Box::new(active_state));
    }

    fn suspend(&mut self) {
        self.state = ShaderState::Suspended;
    }

    fn render(
        &mut self,
        ctx: CustomPaintCtx<'_>,
        width: u32,
        height: u32,
        _scale: f64,
    ) -> Option<TextureHandle> {
        self.process_messages();
        self.render_shader(ctx, width, height)
    }
}

#[allow(dead_code)] // Shader background system - complete implementation pending UI integration
impl ActiveShaderRenderer {
    pub fn new(device: &Device, queue: &Queue) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Background Shader"),
            source: ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "../shaders/background.wgsl"
            ))),
        });

        // Create bind group layout for time uniform
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Time Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Background Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Background Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(TextureFormat::Rgba8Unorm.into())],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create uniform buffer for time
        let time_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Time Uniform Buffer"),
            size: 16, // f32 + 3 * f32 padding for alignment
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group for time uniform
        let time_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Time Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: time_buffer.as_entire_binding(),
            }],
        });

        Self {
            device: device.clone(),
            queue: queue.clone(),
            pipeline,
            displayed_texture: None,
            next_texture: None,
            time_buffer,
            time_bind_group,
        }
    }

    pub fn render(
        &mut self,
        mut ctx: CustomPaintCtx<'_>,
        width: u32,
        height: u32,
        start_time: &Instant,
    ) -> Option<TextureHandle> {
        // If "next texture" size doesn't match specified size then unregister and drop texture
        if let Some(next) = &self.next_texture
            && (next.texture.width() != width || next.texture.height() != height)
        {
            ctx.unregister_texture(next.handle.clone());
            self.next_texture = None;
        }

        // If there is no "next texture" then create one and register it.
        let texture_and_handle = match &self.next_texture {
            Some(next) => next,
            None => {
                let texture = create_texture(&self.device, width, height);
                let texture_clone = texture.clone();
                let handle = ctx.register_texture(texture);
                self.next_texture = Some(TextureAndHandle {
                    texture: texture_clone,
                    handle,
                });
                // Safe unwrap: we just set next_texture to Some() above
                self.next_texture
                    .as_ref()
                    .expect("Texture creation just succeeded")
            }
        };

        let next_texture = &texture_and_handle.texture;
        let next_texture_handle = texture_and_handle.handle.clone();

        let elapsed: f32 = start_time.elapsed().as_millis() as f32 / 1000.0;

        // Update time uniform buffer
        let time_data: [f32; 4] = [elapsed, 0.0, 0.0, 0.0]; // time + padding for alignment
        self.queue
            .write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&time_data));

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Background Render Encoder"),
            });
        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Background Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &next_texture.create_view(&TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.18,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.time_bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));

        std::mem::swap(&mut self.next_texture, &mut self.displayed_texture);
        Some(next_texture_handle)
    }
}

#[allow(dead_code)] // Shader background system - complete implementation pending UI integration
fn create_texture(device: &Device, width: u32, height: u32) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("Background Texture"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

#[component]
pub fn ShaderBackground() -> Element {
    // Create custom paint source and register it with the renderer
    let paint_source = ShaderPaintSource::new();
    let sender = paint_source.sender();
    let paint_source_id = use_wgpu(move || paint_source);

    use_effect(move || {
        // Set up animation timer using use_future for cross-platform compatibility
        let sender_clone = sender.clone();

        // Start animation loop
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16)); // ~60 FPS

            loop {
                interval.tick().await;

                // Send animation frame update
                if sender_clone.send(ShaderMessage::UpdateTime(0.0)).is_err() {
                    break; // Channel closed, stop animation
                }
            }
        });
    });

    rsx!(
        canvas {
            id: "shader-background",
            style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: -1; pointer-events: none;",
            "src": paint_source_id
        }
    )
}
