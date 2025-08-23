use anyhow::{Context, Result};
use crossbeam_channel::Receiver;
use wayland_client::{Connection, EventQueue};
use smithay_client_toolkit::shell::WaylandSurface;

use crate::config::Config;
use crate::terminal::Terminal;
use crate::wayland::WaylandState;

pub struct Display {
    wayland_state: WaylandState,
    connection: Connection,
    event_queue: EventQueue<WaylandState>,
    event_receiver: Option<Receiver<Event>>,
}

#[derive(Debug)]
pub enum Event {
    Resize(u32, u32),
    Key(crate::input::Key),
    Close,
}

impl Display {
    pub async fn new(config: &Config) -> Result<Self> {
        let (mut wayland_state, connection, mut event_queue) = 
            WaylandState::new(config).context("Failed to create Wayland state")?;
            
        let qh = event_queue.handle();
        wayland_state.create_window(&qh).context("Failed to create window")?;
        
        // Process initial events to set up the window
        event_queue.roundtrip(&mut wayland_state)
            .context("Failed to process initial events")?;
        
        Ok(Self {
            wayland_state,
            connection,
            event_queue,
            event_receiver: None,
        })
    }
    
    pub async fn next_event(&mut self) -> Result<Event> {
        loop {
            // Process Wayland events
            if let Err(e) = self.event_queue.dispatch_pending(&mut self.wayland_state) {
                // Handle dispatch errors appropriately
                return Err(e.into());
            }
            
            // Check for exit condition
            if self.wayland_state.should_exit() {
                return Ok(Event::Close);
            }
            
            // Wait for more events
            self.connection.flush().context("Failed to flush connection")?;
            
            match self.event_queue.prepare_read() {
                Some(guard) => {
                    guard.read().context("Failed to read events")?;
                }
                None => {
                    // Events are pending, process them in the next loop iteration
                    continue;
                }
            }
        }
    }
    
    pub async fn render(&mut self, terminal: &Terminal) -> Result<()> {
        // For now, this is a stub. In a complete implementation, this would:
        // 1. Create a shared memory buffer
        // 2. Render the terminal grid to the buffer using font rendering
        // 3. Attach the buffer to the surface and commit
        
        log::debug!("Rendering terminal with {} rows, {} columns", 
                   terminal.grid().rows, terminal.grid().cols);
        
        // Commit any pending changes to the surface
        if let Some(ref window) = self.wayland_state.window {
            window.wl_surface().commit();
        }
        
        Ok(())
    }
    
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        log::debug!("Display resize: {}x{}", width, height);
        Ok(())
    }
}