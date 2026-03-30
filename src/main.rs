mod terminal;
mod renderer;

use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{Read, Write};
use portable_pty::{native_pty_system, PtySize, CommandBuilder, MasterPty};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

use crate::terminal::Terminal;
use crate::renderer::Renderer;

struct App {
    terminal: Arc<Mutex<Terminal>>,
    renderer: Option<Renderer>,
    window: Option<Arc<Window>>,
    pty_master: Box<dyn MasterPty + Send>,
    pty_writer: Arc<Mutex<Box<dyn Write + Send>>>,
    redraw_pending: Arc<std::sync::atomic::AtomicBool>,
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_title("myterm")
            .with_transparent(true);

        #[cfg(target_os = "linux")]
        {
            use winit::platform::wayland::WindowAttributesExtWayland;
            window_attributes = window_attributes.with_name("myterm", "myterm");
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(Renderer::new(window));
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                    
                    let cols = (size.width / 10).max(1) as u16; // Dummy cell size
                    let rows = (size.height / 20).max(1) as u16;
                    
                    self.pty_master.resize(PtySize {
                        rows,
                        cols,
                        pixel_width: size.width as u16,
                        pixel_height: size.height as u16,
                    }).unwrap();
                    
                    let mut term = self.terminal.lock().unwrap();
                    term.resize(rows as usize, cols as usize);
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let start = std::time::Instant::now();
                if let (Some(renderer), Some(_window)) = (&mut self.renderer, &self.window) {
                    let mut term = self.terminal.lock().unwrap();
                    renderer.render(&mut term);
                }
                let duration = start.elapsed();
                log::trace!("Render took {:?}", duration);
                if duration.as_millis() > 16 {
                    log::warn!("Slow frame: {:?}", duration);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() {
                    // Send keys to PTY
                    let mut writer = self.pty_writer.lock().unwrap();
                    if let Some(text) = event.text {
                        writer.write_all(text.as_bytes()).unwrap();
                    } else {
                        // Handle special keys like Enter, Arrows, etc.
                        match event.logical_key {
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter) => {
                                writer.write_all(b"\r").unwrap();
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace) => {
                                writer.write_all(b"\x7f").unwrap();
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab) => {
                                writer.write_all(b"\t").unwrap();
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                                writer.write_all(b"\x1b[A").unwrap();
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                                writer.write_all(b"\x1b[B").unwrap();
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => {
                                writer.write_all(b"\x1b[C").unwrap();
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft) => {
                                writer.write_all(b"\x1b[D").unwrap();
                            }
                            _ => {}
                        }
                    }
                    writer.flush().unwrap();
                }
            }
            _ => (),
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {
        self.redraw_pending.store(false, std::sync::atomic::Ordering::SeqCst);
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    }).unwrap();

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
    let cmd = CommandBuilder::new(shell);
    let _child = pair.slave.spawn_command(cmd).unwrap();

    let terminal = Arc::new(Mutex::new(Terminal::new(24, 80)));
    let terminal_clone = terminal.clone();

    let mut reader = pair.master.try_clone_reader().unwrap();
    let pty_writer = Arc::new(Mutex::new(pair.master.take_writer().unwrap()));
    
    let event_loop = EventLoop::<()>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();
    let redraw_pending = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let redraw_pending_clone = redraw_pending.clone();

    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        while let Ok(n) = reader.read(&mut buffer) {
            if n == 0 { break; }
            {
                let mut term = terminal_clone.lock().unwrap();
                term.advance(&buffer[..n]);
            }
            if !redraw_pending_clone.swap(true, std::sync::atomic::Ordering::SeqCst) {
                let _ = proxy.send_event(());
            }
        }
    });

    event_loop.set_control_flow(ControlFlow::Wait);
    
    let mut app = App {
        terminal,
        renderer: None,
        window: None,
        pty_master: pair.master,
        pty_writer,
        redraw_pending,
    };

    event_loop.run_app(&mut app).unwrap();
}
