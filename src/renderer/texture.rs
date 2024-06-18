use std::sync::Arc;

use super::helpers::{CacheEntry, Cacheable, Fingerprint};
use image::{DynamicImage, RgbaImage};

/// A texture.
#[derive(Debug, Clone)]
pub enum Texture {
    /// A texture backed by a RGBA image.
    RgbaImageTexture {
        /// The image data.
        image: Arc<RgbaImage>,
        /// The internal id for caching.
        id: CacheEntry,
        /// The fingerprint. Changes when the image changes.
        fingerprint: u64,
    },
    /// A texture backed by a raw buffer.
    RawTexture {
        /// The raw buffer.
        buffer: Vec<u8>,
        /// The width of the texture.
        width: u32,
        /// The height of the texture.
        height: u32,
        /// The internal id for caching.
        id: CacheEntry,
        /// The fingerprint. Changes when the buffer changes.
        fingerprint: u64,
    },
}

impl Texture {
    /// New texture from a RGBA image.
    pub fn new_from_image(image: DynamicImage) -> Self {
        let image = image.into_rgba8();
        Self::RgbaImageTexture {
            image: Arc::new(image),
            id: CacheEntry::new(),
            fingerprint: rand::random(),
        }
    }

    pub fn update_image(&mut self, image: DynamicImage) {
        match self {
            Self::RgbaImageTexture {
                image: old_image,
                fingerprint,
                ..
            } => {
                let new_image = image.into_rgba8();
                if old_image.dimensions() == new_image.dimensions() {
                    *fingerprint = rand::random();
                    *old_image = Arc::new(new_image);
                } else {
                    panic!("Image dimensions do not match");
                }
            }
            _ => panic!("Texture is not backed by an image"),
        }
    }

    /// New texture from a cache entry.
    /// Returns the size of the texture.
    pub fn size(&self) -> (u32, u32) {
        match self {
            Self::RgbaImageTexture { image, .. } => image.dimensions(),
            Self::RawTexture { width, height, .. } => (*width, *height),
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            Self::RgbaImageTexture { image, .. } => image.width(),
            Self::RawTexture { width, .. } => *width,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Self::RgbaImageTexture { image, .. } => image.height(),
            Self::RawTexture { height, .. } => *height,
        }
    }
    /// Returns the image data as a byte slice.
    pub fn data(&self) -> &[u8] {
        match self {
            Self::RgbaImageTexture { image, .. } => &image,
            Self::RawTexture { buffer, .. } => buffer,
        }
    }
}

impl Fingerprint for Texture {
    fn fingerprint(&self) -> u64 {
        match self {
            Self::RgbaImageTexture { fingerprint, .. } => *fingerprint,
            Self::RawTexture { fingerprint, .. } => *fingerprint,
        }
    }
}

impl Cacheable for Texture {
    fn cache_id(&self) -> CacheEntry {
        match self {
            Self::RgbaImageTexture { id, .. } => id.clone(),
            Self::RawTexture { id, .. } => id.clone(),
        }
    }
}
