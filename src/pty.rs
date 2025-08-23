use anyhow::{Context, Result};
use nix::pty::{openpty, Winsize};
use nix::sys::signal::{self, Signal};
use nix::unistd::{close, dup2, execve, fork, setsid, ForkResult, Pid};
use std::ffi::CString;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::process;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Pty {
    master_fd: RawFd,
    slave_fd: RawFd,
    child_pid: Option<Pid>,
    master_file: Option<File>,
}

impl Pty {
    pub fn new() -> Result<Self> {
        let pty_result = openpty(None, None)?;
        
        // Convert OwnedFd to RawFd for compatibility
        let master_fd = pty_result.master.as_raw_fd();
        let slave_fd = pty_result.slave.as_raw_fd();
        
        // Prevent automatic closing of file descriptors
        std::mem::forget(pty_result.master);
        std::mem::forget(pty_result.slave);
        
        Ok(Self {
            master_fd,
            slave_fd,
            child_pid: None,
            master_file: None,
        })
    }
    
    pub async fn spawn_shell(&mut self, shell: Option<&str>, working_dir: Option<&str>) -> Result<()> {
        let default_shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let shell = shell.unwrap_or(&default_shell);
        
        match unsafe { fork() }? {
            ForkResult::Parent { child } => {
                self.child_pid = Some(child);
                close(self.slave_fd)?;
                
                let master_file = unsafe {
                    File::from_raw_fd(self.master_fd)
                };
                self.master_file = Some(master_file);
                
                Ok(())
            }
            ForkResult::Child => {
                setsid()?;
                
                close(self.master_fd)?;
                
                dup2(self.slave_fd, 0)?; // stdin
                dup2(self.slave_fd, 1)?; // stdout  
                dup2(self.slave_fd, 2)?; // stderr
                
                if self.slave_fd > 2 {
                    close(self.slave_fd)?;
                }
                
                if let Some(dir) = working_dir {
                    std::env::set_current_dir(dir)
                        .context("Failed to set working directory")?;
                }
                
                let shell_cstr = CString::new(shell)?;
                let args = [&shell_cstr];
                let env_vars: Vec<CString> = std::env::vars()
                    .map(|(key, value)| CString::new(format!("{}={}", key, value)))
                    .collect::<Result<Vec<_>, _>>()?;
                
                execve(&shell_cstr, &args, &env_vars)?;
                
                process::exit(1);
            }
        }
    }
    
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if let Some(ref mut file) = self.master_file {
            Ok(file.read(buf).await?)
        } else {
            Err(anyhow::anyhow!("PTY not initialized"))
        }
    }
    
    pub async fn write(&mut self, data: &[u8]) -> Result<()> {
        if let Some(ref mut file) = self.master_file {
            file.write_all(data).await?;
            file.flush().await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("PTY not initialized"))
        }
    }
    
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let winsize = Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        
        // Use nix's built-in TIOCSWINSZ functionality
        use nix::ioctl_write_ptr_bad;
        ioctl_write_ptr_bad!(tiocswinsz, libc::TIOCSWINSZ, Winsize);
        
        unsafe {
            tiocswinsz(self.master_fd, &winsize)?;
        }
        
        Ok(())
    }
    
    pub fn child_pid(&self) -> Option<Pid> {
        self.child_pid
    }
    
    pub fn send_signal(&self, sig: Signal) -> Result<()> {
        if let Some(pid) = self.child_pid {
            signal::kill(pid, sig)?;
        }
        Ok(())
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        if let Some(pid) = self.child_pid {
            let _ = signal::kill(pid, Signal::SIGTERM);
        }
        
        if self.master_file.is_none() {
            let _ = close(self.master_fd);
        }
    }
}