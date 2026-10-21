//! Decoded card images, in the form a PDF image needs.

use image::GenericImageView;

/// A card image decoded into separate 8-bit RGB and alpha planes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// RGB samples, row by row.
    pub rgb: Vec<u8>,
    /// Alpha samples, present only when some pixel is not fully opaque.
    pub alpha: Option<Vec<u8>>,
}

impl CardImage {
    /// Decodes an encoded image (PNG or JPEG).
    ///
    /// # Errors
    /// Returns [`image::ImageError`] if the format is unsupported or the data is corrupt.
    pub fn decode(bytes: &[u8]) -> Result<Self, image::ImageError> {
        let image = image::load_from_memory(bytes)?;
        let (width, height) = image.dimensions();
        let alpha = image
            .color()
            .has_alpha()
            .then(|| {
                image
                    .to_rgba8()
                    .pixels()
                    .map(|pixel| pixel.0[3])
                    .collect::<Vec<_>>()
            })
            .filter(|alpha| alpha.iter().any(|&a| a != u8::MAX));
        Ok(Self {
            width,
            height,
            rgb: image.to_rgb8().into_raw(),
            alpha,
        })
    }

    /// Height-to-width ratio, used to scale the image to a given width.
    pub fn aspect_ratio(&self) -> f32 {
        self.height as f32 / self.width as f32
    }
}
