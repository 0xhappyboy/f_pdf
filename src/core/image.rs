//! Image structures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FColorSpace {
    /// DeviceGray.
    DeviceGray,
    /// DeviceRGB.
    DeviceRGB,
    /// DeviceCMYK.
    DeviceCMYK,
    /// Indexed color space.
    Indexed,
    /// Separation color space.
    Separation,
    /// DeviceN color space.
    DeviceN,
    /// ICCBased color space.
    ICCBased,
    /// Pattern color space.
    Pattern,
    /// Lab color space.
    Lab,
    /// CalGray color space.
    CalGray,
    /// CalRGB color space.
    CalRGB,
    /// Named color space.
    Named(String),
}
impl Default for FColorSpace {
    fn default() -> Self {
        FColorSpace::DeviceRGB
    }
}
/// Image interpolation flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FImageInterpolation {
    /// No interpolation.
    #[default]
    None,
    /// Interpolate when scaling.
    Interpolate,
}
/// An image XObject.
#[derive(Debug, Clone, Default)]
pub struct FImage {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Number of bits per component.
    pub bits_per_component: u32,
    /// Color space of the image.
    pub color_space: FColorSpace,
    /// Image data.
    pub data: Vec<u8>,
    /// Whether the image is compressed.
    pub compressed: bool,
    /// Compression filter.
    pub filter: Option<String>,
    /// Interpolation flag.
    pub interpolation: FImageInterpolation,
    /// Image mask flag (1-bit transparency mask).
    pub is_mask: bool,
    /// Soft mask (transparency) image reference.
    pub soft_mask: Option<u32>,
    /// Decode array (color mapping).
    pub decode: Option<Vec<f32>>,
    /// Raw image dictionary.
    pub raw_dict: Option<String>,
}
/// An image mask.
#[derive(Debug, Clone, Default)]
pub struct FImageMask {
    /// Mask width in pixels.
    pub width: u32,
    /// Mask height in pixels.
    pub height: u32,
    /// Mask data (1 bit per pixel).
    pub data: Vec<u8>,
    /// Raw mask dictionary.
    pub raw_dict: Option<String>,
}
