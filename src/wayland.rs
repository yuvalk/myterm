use anyhow::{Context, Result};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Modifiers as WaylandModifiers},
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        xdg::{
            window::{Window, WindowConfigure, WindowHandler},
            XdgShell, XdgSurface,
        },
        WaylandSurface,
    },
    shm::{Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface},
    Connection, QueueHandle,
};

use crate::config::Config;
use crate::input::{Key, KeyCode, Modifiers};

pub struct WaylandState {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    compositor_state: CompositorState,
    shm: Shm,
    xdg_shell: XdgShell,
    
    pub window: Option<Window>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    pointer: Option<wl_pointer::WlPointer>,
    
    exit: bool,
    width: u32,
    height: u32,
    
    event_sender: crossbeam_channel::Sender<Event>,
}

#[derive(Debug)]
pub enum Event {
    #[allow(dead_code)]
    Resize(u32, u32),
    #[allow(dead_code)]
    Key(Key),
    Close,
}

impl WaylandState {
    pub fn new(config: &Config) -> Result<(Self, Connection, wayland_client::EventQueue<Self>)> {
        let (event_sender, _) = crossbeam_channel::unbounded();
        
        let conn = Connection::connect_to_env()
            .context("Failed to connect to Wayland display")?;
        
        let (globals, event_queue) = registry_queue_init(&conn)
            .context("Failed to initialize registry")?;
            
        let qh = event_queue.handle();
        
        let compositor_state = CompositorState::bind(&globals, &qh)
            .context("Failed to bind compositor")?;
        let xdg_shell = XdgShell::bind(&globals, &qh)
            .context("Failed to bind XDG shell")?;
        let shm = Shm::bind(&globals, &qh)
            .context("Failed to bind shared memory")?;
        
        let registry_state = RegistryState::new(&globals);
        let seat_state = SeatState::new(&globals, &qh);
        let output_state = OutputState::new(&globals, &qh);
        
        let state = Self {
            registry_state,
            seat_state,
            output_state,
            compositor_state,
            shm,
            xdg_shell,
            window: None,
            keyboard: None,
            pointer: None,
            exit: false,
            width: config.display.width,
            height: config.display.height,
            event_sender,
        };
        
        Ok((state, conn, event_queue))
    }
    
    pub fn create_window(&mut self, qh: &QueueHandle<Self>) -> Result<()> {
        log::debug!("Creating Wayland window");
        let surface = self.compositor_state.create_surface(qh);
        log::debug!("Created surface");
        
        let window = self.xdg_shell.create_window(
            surface, 
            smithay_client_toolkit::shell::xdg::window::WindowDecorations::RequestServer, 
            qh
        );
        log::debug!("Created XDG window");
        
        window.set_title("MyTerm");
        window.set_app_id("myterm");
        
        // Set initial window size
        window.set_min_size(Some((400, 300)));
        window.set_max_size(Some((2000, 1500)));
        
        log::debug!("Committing window configuration");
        window.commit();
        
        self.window = Some(window);
        log::debug!("Window creation completed");
        Ok(())
    }
    
    pub fn should_exit(&self) -> bool {
        self.exit
    }
    
    fn wayland_key_to_key(&self, event: &KeyEvent, modifiers: &WaylandModifiers) -> Option<Key> {
        let key_modifiers = self.modifiers_to_key_modifiers(modifiers);
        
        match event.utf8 {
            Some(ref text) if !text.is_empty() && !text.chars().all(|c| c.is_control()) => {
                if let Some(c) = text.chars().next() {
                    return Some(Key::new(KeyCode::Char(c), key_modifiers));
                }
            }
            _ => {}
        }
        
        let keycode = match event.raw_code {
            9 => Some(KeyCode::Escape),
            22 => Some(KeyCode::Backspace),
            23 => Some(KeyCode::Tab),
            36 => Some(KeyCode::Enter),
            110 => Some(KeyCode::Home),
            115 => Some(KeyCode::End),
            112 => Some(KeyCode::PageUp),
            117 => Some(KeyCode::PageDown),
            111 => Some(KeyCode::Up),
            116 => Some(KeyCode::Down),
            113 => Some(KeyCode::Left),
            114 => Some(KeyCode::Right),
            119 => Some(KeyCode::Delete),
            118 => Some(KeyCode::Insert),
            67..=76 => Some(KeyCode::F((event.raw_code - 66) as u8)),
            _ => None,
        };
        
        keycode.map(|code| Key::new(code, key_modifiers))
    }
    
    fn modifiers_to_key_modifiers(&self, modifiers: &WaylandModifiers) -> Modifiers {
        let mut key_modifiers = Modifiers::empty();
        
        if modifiers.ctrl {
            key_modifiers.insert(Modifiers::CTRL);
        }
        if modifiers.alt {
            key_modifiers.insert(Modifiers::ALT);
        }
        if modifiers.shift {
            key_modifiers.insert(Modifiers::SHIFT);
        }
        if modifiers.logo {
            key_modifiers.insert(Modifiers::SUPER);
        }
        
        key_modifiers
    }
}

impl CompositorHandler for WaylandState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for WaylandState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl WindowHandler for WaylandState {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
        self.exit = true;
        let _ = self.event_sender.send(Event::Close);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _window: &Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        log::debug!("Window configure event: {:?}", configure);
        
        if let (Some(width), Some(height)) = configure.new_size {
            self.width = width.get();
            self.height = height.get();
            log::debug!("New window size: {}x{}", self.width, self.height);
            let _ = self.event_sender.send(Event::Resize(self.width, self.height));
        } else {
            // Use default size if none specified  
            self.width = 800;
            self.height = 600;
            log::debug!("Using default window size: {}x{}", self.width, self.height);
        }
        
        log::debug!("Window configured");
    }
}

impl SeatHandler for WaylandState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            self.keyboard = Some(
                self.seat_state
                    .get_keyboard(qh, &seat, None)
                    .expect("Failed to create keyboard"),
            );
        }

        if capability == Capability::Pointer && self.pointer.is_none() {
            self.pointer = Some(
                self.seat_state
                    .get_pointer(qh, &seat)
                    .expect("Failed to create pointer"),
            );
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            self.keyboard.take().unwrap().release();
        }

        if capability == Capability::Pointer && self.pointer.is_some() {
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for WaylandState {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _: &[smithay_client_toolkit::seat::keyboard::Keysym],
    ) {
        if Some(surface) == self.window.as_ref().map(|w| w.wl_surface()) {
            // Window gained focus
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if Some(surface) == self.window.as_ref().map(|w| w.wl_surface()) {
            // Window lost focus
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
        // We'll use empty modifiers for now - proper modifier tracking would require
        // storing the current modifier state
        let modifiers = WaylandModifiers::default();
        if let Some(key) = self.wayland_key_to_key(&event, &modifiers) {
            let _ = self.event_sender.send(Event::Key(key));
        }
    }

    fn release_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _event: KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _modifiers: WaylandModifiers,
        _layout: u32,
    ) {
    }
}

impl PointerHandler for WaylandState {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            match &event.kind {
                PointerEventKind::Enter { .. } => {}
                PointerEventKind::Leave { .. } => {}
                PointerEventKind::Motion { .. } => {}
                PointerEventKind::Press { button, .. } => {
                    // Handle mouse button press
                    log::debug!("Mouse button press: {}", button);
                }
                PointerEventKind::Release { button, .. } => {
                    // Handle mouse button release  
                    log::debug!("Mouse button release: {}", button);
                }
                PointerEventKind::Axis { .. } => {
                    // Handle scroll wheel
                }
            }
        }
    }
}

impl ShmHandler for WaylandState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for WaylandState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(WaylandState);
delegate_output!(WaylandState);
delegate_shm!(WaylandState);
delegate_xdg_shell!(WaylandState);
delegate_xdg_window!(WaylandState);
delegate_registry!(WaylandState);
delegate_seat!(WaylandState);
delegate_keyboard!(WaylandState);
delegate_pointer!(WaylandState);