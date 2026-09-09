use anyhow::{Context, Result};
use tokio::{
    io::AsyncReadExt, net::{TcpListener, TcpStream}, process::Command,
};

use crate::modules::display::DisplayIf;

const SOCKET_ADDR: &str = "127.0.0.1:12345";
const EMULATOR: &str = "/root/scripts/j2me/freej2me-sdl.jar";
const FRAME_SIZE: usize = 240 * 320 * 2;

pub struct Emulator {
    proc: tokio::process::Child,
    listener: TcpListener,
    stream: Option<TcpStream>,
}

impl Emulator {
    pub async fn new(path: &str) -> Result<Self> {
        let listener = TcpListener::bind(SOCKET_ADDR)
            .await
            .context("Failed to create display socket")?;

        let proc = Command::new("java")
            .arg("-jar")
            .arg(EMULATOR)
            .arg(path)
            .arg("240")
            .arg("320")
            .spawn()
            .context("Failed to start J2ME emulator")?;

        Ok(Self { proc, listener, stream: None })
    }

    pub async fn poll(&mut self, display: &mut DisplayIf) -> Result<()> {
        let mut buf = vec![0u8; FRAME_SIZE];

        loop {
            if self.stream.is_none() {
                tokio::select! {
                    status = self.proc.wait() => {
                        let status = status.context("Emulator error")?;
                        println!("Emulator exited: {status}");
                        break;
                    }
                    accepted = self.listener.accept() => {
                        let (stream, _addr) = accepted.context("Failed to accept connection")?;
                        self.stream = Some(stream);
                    }
                }
                continue;
            }

            let stream = self.stream.as_mut().unwrap();

            tokio::select! {
                status = self.proc.wait() => {
                    let status = status.context("Emulator error")?;
                    println!("Emulator exited: {status}");
                    break;
                }

                result = stream.read_exact(&mut buf) => {
                    match result {
                        Ok(_) => {
                            let frame = buf.clone();
                            display.draw_img(0, 0, 240, frame).await?;
                            display.flush().await?;
                        }
                        Err(e) => {
                            eprintln!("Frame read error: {e}");
                            self.stream = None;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Drop for Emulator {
    fn drop(&mut self) {
        let _ = self.proc.start_kill();
    }
}
