//! `ImageTag`: an in-memory image (minimal: dimensions + raw RGBA buffer).

use crate::obj::{ObjectTag, ObjectTagImpl};
use std::any::Any;
use std::sync::Arc;

#[derive(Clone)]
pub struct ImageTag {
    pub width: u32,
    pub height: u32,
    pub pixels: Arc<Vec<u8>>,
    pub prefix: String,
}

impl ImageTag {
    pub fn new(width: u32, height: u32) -> Self {
        ImageTag {
            width,
            height,
            pixels: Arc::new(vec![0u8; (width as usize) * (height as usize) * 4]),
            prefix: "image".to_string(),
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        if x < self.width && y < self.height {
            let idx = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
            let px = Arc::make_mut(&mut self.pixels);
            px[idx] = r;
            px[idx + 1] = g;
            px[idx + 2] = b;
            px[idx + 3] = a;
        }
    }
}

impl ObjectTagImpl for ImageTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        format!("image@{}x{}", self.width, self.height)
    }
    fn object_type_name(&self) -> &'static str {
        "ImageTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}
