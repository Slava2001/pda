use std::{
    fs::{self, File},
    io::Read,
};
use anyhow::{Context, Result, bail, ensure};
use png::ColorType;
use zip::ZipArchive;

#[derive(Clone)]
pub struct Game {
    pub path: String,
    pub name: String,
    pub icon: Option<(Vec<u8>, usize)>,
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn load_games(path: &str) -> Result<Vec<Game>> {
    let jars = get_games_list(path)?;
    let mut result = Vec::new();
    for jar in jars {
        result.push(Game {
            name: get_game_name(&jar).unwrap_or(jar.clone()),
            icon: get_game_icon(&jar).ok(),
            path: jar,
        });
    }
    Ok(result)
}

fn get_games_list(path: &str) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(path).context("Failed to read game directory")? {
        let path = entry?.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
        {
            result.push(path.to_string_lossy().into_owned());
        }
    }
    ensure!(!result.is_empty(), "Games not found");
    Ok(result)
}

fn get_game_name(path: &str) -> Result<String> {
    let file = File::open(path).with_context(|| format!("Failed to open {:?}", path))?;
    let mut archive = ZipArchive::new(file).context("Failed to open JAR as ZIP")?;
    let mut manifest = archive
        .by_name("META-INF/MANIFEST.MF")
        .context("MANIFEST.MF not found")?;
    let mut text = String::new();
    manifest.read_to_string(&mut text)?;
    text.lines()
        .find_map(|line| {
            line.strip_prefix("MIDlet-Name:")
                .map(str::trim)
                .map(String::from)
        })
        .context("MIDlet-Name not found")
}

fn get_game_icon(path: &str) -> Result<(Vec<u8>, usize)> {
    let file = File::open(path).with_context(|| format!("Failed to open {:?}", path))?;
    let mut archive = ZipArchive::new(file).context("Failed to open JAR as ZIP")?;
    let mut manifest = archive
        .by_name("META-INF/MANIFEST.MF")
        .context("MANIFEST.MF not found")?;
    let mut text = String::new();
    manifest.read_to_string(&mut text)?;
    let icon_path = text
        .lines()
        .find_map(|line| line.strip_prefix("MIDlet-Icon:").map(str::trim))
        .context("MIDlet-Icon not found")?
        .trim_start_matches('/');
    drop(manifest);
    let mut icon = archive
        .by_name(icon_path)
        .context("Icon file not found in JAR")?;

    let mut png_data = Vec::new();
    icon.read_to_end(&mut png_data)?;

    let decoder = png::Decoder::new(std::io::Cursor::new(&png_data));
    let mut reader = decoder.read_info()?;

    let mut pixels = vec![
        0;
        reader
            .output_buffer_size()
            .context("Failed to get buffer_size")?
    ];
    let info = reader.next_frame(&mut pixels)?;

    pixels.truncate(info.buffer_size());

    let rgb565 = match info.color_type {
        ColorType::Rgb => rgb888_to_rgb565(&pixels),
        ColorType::Rgba => rgba8888_to_rgb565(&pixels),
        ColorType::Grayscale => grayscale_to_rgb565(&pixels),
        color_type => bail!("Unsupported icon format: {color_type:?}"),
    };
    Ok((rgb565, info.width as usize))
}

fn grayscale_to_rgb565(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() * 2);
    for &v in data {
        let v = v as u16;
        let color = ((v >> 3) << 11) | ((v >> 2) << 5) | (v >> 3);
        out.push((color >> 8) as u8);
        out.push(color as u8);
    }
    out
}

fn rgb888_to_rgb565(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() / 2);
    for pixel in data.chunks_exact(3) {
        let r = pixel[0] as u16;
        let g = pixel[1] as u16;
        let b = pixel[2] as u16;
        let color = ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3);
        out.push((color >> 8) as u8);
        out.push(color as u8);
    }
    out
}

fn rgba8888_to_rgb565(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() / 2);
    for pixel in data.chunks_exact(4) {
        let a = pixel[3] as u16;
        let r = pixel[0] as u16 * a / 255;
        let g = pixel[1] as u16 * a / 255;
        let b = pixel[2] as u16 * a / 255;
        let color = ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3);
        out.push((color >> 8) as u8);
        out.push(color as u8);
    }
    out
}
