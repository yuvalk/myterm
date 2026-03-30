use std::sync::Arc;
use wgpu::*;
use glyphon::{
    FontSystem, SwashCache, TextAtlas, TextRenderer, TextArea, Metrics, Shaping,
    Viewport, Resolution, Color as TextColor, Cache,
};
use crate::terminal::Terminal;

pub struct Renderer {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub atlas: TextAtlas,
    pub renderer: TextRenderer,
    pub viewport: Viewport,
    pub _cache: Cache,
    pub text_buffer: glyphon::Buffer,
}

impl Renderer {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let instance = Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await.unwrap();

        let size = window.inner_size();
        let config = surface.get_default_config(&adapter, size.width, size.height).unwrap();
        surface.configure(&device, &config);

        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, config.format);
        let renderer = TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);

        let mut viewport = Viewport::new(&device, &cache);
        viewport.update(&queue, Resolution {
            width: config.width,
            height: config.height,
        });

        let text_buffer = glyphon::Buffer::new(&mut font_system, Metrics::new(14.0, 18.0));

        Self {
            device,
            queue,
            surface,
            config,
            font_system,
            swash_cache,
            atlas,
            renderer,
            viewport,
            _cache: cache,
            text_buffer,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.viewport.update(&self.queue, Resolution { width, height });
            self.text_buffer.set_size(&mut self.font_system, Some(width as f32), Some(height as f32));
        }
    }

    pub fn render(&mut self, terminal: &mut Terminal) {
        if terminal.state.is_dirty {
            let mut full_text = String::with_capacity(terminal.state.rows * (terminal.state.cols + 1));
            for r in 0..terminal.state.rows {
                for c in 0..terminal.state.cols {
                    full_text.push(terminal.state.get_cell(r, c).c);
                }
                full_text.push('\n');
            }
            
            self.text_buffer.set_text(&mut self.font_system, &full_text, &glyphon::Attrs::new().family(glyphon::Family::Monospace), Shaping::Advanced, None);
            terminal.state.is_dirty = false;
        }

        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to get current texture: {:?}", e);
                return;
            }
        };
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        let text_areas = [TextArea {
            buffer: &self.text_buffer,
            left: 5.0,
            top: 5.0,
            scale: 1.0,
            bounds: glyphon::TextBounds {
                left: 0,
                top: 0,
                right: self.config.width as i32,
                bottom: self.config.height as i32,
            },
            default_color: TextColor::rgb(255, 255, 255),
            custom_glyphs: &[],
        }];

        self.renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        ).unwrap();

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color { r: 0.01, g: 0.01, b: 0.02, a: 0.9 }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            
            self.renderer.render(&self.atlas, &self.viewport, &mut render_pass).unwrap();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
