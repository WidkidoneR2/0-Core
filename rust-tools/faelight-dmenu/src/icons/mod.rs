//! Icon loading and caching system
use image::{RgbaImage, imageops::FilterType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct IconCache {
    icons: HashMap<String, RgbaImage>,
    default_icon: RgbaImage,
    icon_size: u32,
}

impl IconCache {
    pub fn new(icon_size: u32) -> Self {
        let default_icon = create_default_icon(icon_size);
        Self {
            icons: HashMap::new(),
            default_icon,
            icon_size,
        }
    }
    
    pub fn load_icon(&mut self, app_id: &str, icon_name: Option<&str>) {
        let name = icon_name.unwrap_or(app_id);
        if let Some(icon_path) = find_icon(name, self.icon_size) {
            if let Some(icon) = load_and_scale_icon(&icon_path, self.icon_size) {
                eprintln!("✅ Loaded icon: {} → {:?}", name, icon_path);
                self.icons.insert(app_id.to_string(), icon);
            }
        } else {
            eprintln!("❌ No icon found for: {}", name);
        }
    }
    
    pub fn get(&self, app_id: &str) -> &RgbaImage {
        self.icons.get(app_id).unwrap_or(&self.default_icon)
    }
}

fn create_default_icon(size: u32) -> RgbaImage {
    // Create a simple default icon (rounded square with app symbol)
    let mut img = RgbaImage::new(size, size);
    
    // Fill with semi-transparent gray
    for pixel in img.pixels_mut() {
        *pixel = image::Rgba([100, 100, 100, 200]);
    }
    
    img
}

fn find_icon(name: &str, size: u32) -> Option<PathBuf> {
    // Icon search paths (in priority order)
    let search_paths = vec![
        format!("/usr/share/icons/hicolor/{}x{}/apps", size, size),
        format!("/usr/share/icons/hicolor/scalable/apps"),
        format!("/usr/share/pixmaps"),
        format!("{}/.local/share/icons/hicolor/{}x{}/apps", 
                std::env::var("HOME").ok()?, size, size),
    ];
    
    // Try different extensions
    let extensions = vec!["png", "svg", "xpm"];
    
    for dir in search_paths {
        for ext in &extensions {
            let path = PathBuf::from(&dir).join(format!("{}.{}", name, ext));
            if path.exists() {
                return Some(path);
            }
        }
    }
    
    None
}

fn load_and_scale_icon(path: &Path, size: u32) -> Option<RgbaImage> {
    // Only support PNG for now (SVG requires another crate)
    if path.extension()?.to_str()? != "png" {
        return None;
    }
    
    let img = image::open(path).ok()?;
    let resized = img.resize_exact(size, size, FilterType::Lanczos3);
    Some(resized.to_rgba8())
}
