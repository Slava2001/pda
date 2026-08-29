use crate::{
    core::{interface::IfMngr, module::Module},
    create_imc_interface,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::{Rgb565, RgbColor},
    primitives::{Primitive, PrimitiveStyle, Rectangle},
    text::Text,
};
use std::sync::Arc;
use tokio::sync::Mutex;

create_imc_interface! {
    pub interface DisplayIf {
        fn draw_line(text: String, line: usize);
        fn text_mode_size()->(usize, usize);
        fn clear();
        fn flush();
    }
}

pub struct Display {
    display: Arc<Mutex<tft::Display>>,
}

impl Display {
    pub async fn build(_if_mngr: IfMngr) -> Result<Self> {
        tft::Display::wait_files().await;
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
                const LINE_SPACING: u32 = 0;
                let font = &FONT_10X20;
                let line_height = font.character_size.height + LINE_SPACING;
                let y = (line as i32) * (line_height as i32);
                Rectangle::new(Point::new(0, y), Size::new(tft::WIDTH as u32, line_height))
                    .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
                    .draw(&mut *display.lock().await)
                    .ok();

                let style = MonoTextStyle::new(font, Rgb565::WHITE);
                Text::new(
                    &text,
                    Point::new(0, y + (font.baseline + LINE_SPACING / 2) as i32),
                    style,
                )
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

        builder = builder.on_text_mode_size(move || async move {
            (
                tft::WIDTH / FONT_10X20.character_size.width as usize,
                tft::HEIGHT / FONT_10X20.character_size.height as usize,
            )
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
    use std::path::Path;
    use std::println;
    use std::time::Duration;
    use std::{fs::OpenOptions, io};
    use tokio::time::sleep;

    const FB_PATH: &str = "/dev/fb0";
    pub const WIDTH: usize = 240;
    pub const HEIGHT: usize = 320;
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
        async fn wait_file(path: impl AsRef<Path>) {
            let path = path.as_ref();
            while !path.exists() {
                println!("File {path:?} not exist, waiting");
                sleep(Duration::from_millis(100)).await;
            }
        }

        pub async fn wait_files() {
            Self::wait_file(TTY_PATH).await;
            Self::wait_file(FB_PATH).await;
        }

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
                self.frame[offset] = (raw & 0xFF) as u8;
                self.frame[offset + 1] = (raw >> 8) as u8;
            }
            Ok(())
        }
    }
}
