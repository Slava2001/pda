use crate::{
    core::{interface::IfMngr, module::Module},
    create_imc_interface,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::{Rgb565, RgbColor},
    text::Text,
};
use std::sync::Arc;
use tokio::sync::Mutex;

create_imc_interface! {
    pub interface DisplayIf {
        fn draw_line(text: String, line: usize);
        fn clear();
        fn flush();
    }
}

pub struct Display {
    display: Arc<Mutex<tft::Display>>,
}

impl Display {
    pub async fn build(_if_mngr: IfMngr) -> Result<Self> {
        Ok(Self {
            display: Arc::new(Mutex::new(
                tft::Display::new().context("Failed to init tft display")?,
            )),
        })
    }
}

#[async_trait]
impl Module for Display {
    async fn run(&mut self, if_mngr: IfMngr) -> Result<()> {
        let mut builder = DisplayIfBackend::builder();

        let display = self.display.clone();
        builder = builder.on_draw_line(move |text, line| {
            let display = display.clone();
            async move {
                let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
                Text::new(&text, Point::new(0, (line as i32 + 1) * 23), style)
                    .draw(&mut *display.lock().await)
                    .ok();
            }
        });

        let display = self.display.clone();
        builder = builder.on_flush(move || {
            let display = display.clone();
            async move {
                display.lock().await.flush();
            }
        });

        let display = self.display.clone();
        builder = builder.on_clear(move || {
            let display = display.clone();
            async move {
                display.lock().await.clear(Rgb565::BLACK).ok();
            }
        });

        let mut display_if = builder
            .build()
            .context("Failed to build display interface")?;

        let frontend = display_if.get_if();
        if_mngr
            .reg("display", move || frontend.clone())
            .await
            .context("Failed to reg display interface")?;

        loop {
            display_if.poll().await;
        }
    }
}

mod tft {
    use embedded_graphics::Pixel;
    use embedded_graphics::draw_target::DrawTarget;
    use embedded_graphics::geometry::{OriginDimensions, Size};
    use embedded_graphics::pixelcolor::{IntoStorage, Rgb565};
    use memmap2::{MmapMut, MmapOptions};
    use std::fs::File;
    use std::os::fd::AsRawFd;
    use std::{fs::OpenOptions, io};

    const FB_PATH: &str = "/dev/fb0";
    const WIDTH: usize = 240;
    const HEIGHT: usize = 320;
    const FB_SIZE: usize = WIDTH * HEIGHT * 2;
    const TTY_PATH: &str = "/dev/tty1";
    const KDSETMODE: u32 = 0x4B3A;
    const KD_GRAPHICS: i32 = 0x01;

    pub struct Display {
        frame: [u8; FB_SIZE],
        fb: MmapMut,
        _tty: File,
    }

    impl Display {
        pub fn new() -> io::Result<Self> {
            let tty = OpenOptions::new().read(true).write(true).open(TTY_PATH)?;
            let ret = unsafe { libc::ioctl(tty.as_raw_fd(), KDSETMODE.into(), KD_GRAPHICS) };
            if ret != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let file = OpenOptions::new().read(true).write(true).open(FB_PATH)?;
            let fb = unsafe { MmapOptions::new().len(FB_SIZE).map_mut(&file)? };
            Ok(Self {
                frame: [0; FB_SIZE],
                fb,
                _tty: tty,
            })
        }

        pub fn flush(&mut self) {
            self.fb.clone_from_slice(&self.frame);
        }
    }

    impl OriginDimensions for Display {
        fn size(&self) -> Size {
            Size::new(WIDTH as u32, HEIGHT as u32)
        }
    }

    impl DrawTarget for Display {
        type Color = Rgb565;
        type Error = std::convert::Infallible;

        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            for Pixel(coord, color) in pixels {
                let x = coord.x;
                let y = coord.y;
                if x < 0 || y < 0 || x as usize >= WIDTH || y as usize >= HEIGHT {
                    continue;
                }
                let offset = (y as usize * WIDTH + x as usize) * 2;
                let raw = color.into_storage();
                self.frame[offset] = (raw >> 8) as u8;
                self.frame[offset + 1] = (raw & 0xFF) as u8;
            }
            Ok(())
        }
    }
}
