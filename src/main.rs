use anyhow::Result;
use log::{debug, info, warn};

mod config;
mod display;
mod input;
mod pty;
mod terminal;
mod wayland;

use config::Config;
use display::Display;
use terminal::Terminal;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    info!("Starting MyTerm - Modern terminal for Sway/Wayland");
    
    let config = Config::load().unwrap_or_else(|e| {
        warn!("Failed to load config: {}, using defaults", e);
        Config::default()
    });
    
    debug!("Configuration loaded: {:?}", config);
    
    let display = Display::new(&config).await?;
    let mut terminal = Terminal::new(&config)?;
    
    terminal.start_shell(&config).await?;
    
    let app = MyTermApp::new(config, display, terminal);
    app.run().await
}

struct MyTermApp {
    #[allow(dead_code)]
    config: Config,
    display: Display,
    terminal: Terminal,
}

impl MyTermApp {
    fn new(config: Config, display: Display, terminal: Terminal) -> Self {
        Self {
            config,
            display,
            terminal,
        }
    }
    
    async fn run(mut self) -> Result<()> {
        info!("MyTerm application started");
        
        loop {
            tokio::select! {
                display_event = self.display.next_event() => {
                    match display_event? {
                        display::Event::Resize(width, height) => {
                            self.terminal.resize(width, height)?;
                            self.display.render(&self.terminal).await?;
                        }
                        display::Event::Key(key) => {
                            let bytes = key.to_bytes();
                            if !bytes.is_empty() {
                                self.terminal.write_to_pty(&bytes).await?;
                            }
                            self.display.render(&self.terminal).await?;
                        }
                        display::Event::Close => {
                            info!("Received close event, shutting down");
                            break;
                        }
                    }
                }
                terminal_output = self.terminal.next_output() => {
                    if let Some(_output) = terminal_output? {
                        self.display.render(&self.terminal).await?;
                    }
                }
            }
        }
        
        info!("MyTerm application shutting down");
        Ok(())
    }
}