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

        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, config.format);
        let renderer = TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);

        let mut viewport = Viewport::new(&device, &cache);
        viewport.update(&queue, Resolution {
            width: config.width,
            height: config.height,
        });

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
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.viewport.update(&self.queue, Resolution { width, height });
        }
    }

    pub fn render(&mut self, terminal: &Terminal) {
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to get current texture: {:?}", e);
                return;
            }
        };
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        let mut text_areas = Vec::new();
        let mut text_buffer = glyphon::Buffer::new(&mut self.font_system, Metrics::new(14.0, 18.0));
        
        let mut full_text = String::new();
        for row in &terminal.grid {
            for cell in row {
                full_text.push(cell.c);
            }
            full_text.push('\n');
        }
        
        text_buffer.set_text(&mut self.font_system, &full_text, &glyphon::Attrs::new().family(glyphon::Family::Monospace), Shaping::Advanced, None);
        text_buffer.set_size(&mut self.font_system, Some(self.config.width as f32), Some(self.config.height as f32));

        text_areas.push(TextArea {
            buffer: &text_buffer,
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
        });

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
