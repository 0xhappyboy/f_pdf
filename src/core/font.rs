//! Font structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FFontType {
    /// Type 1 font.
    Type1,
    /// TrueType font.
    TrueType,
    /// Type 3 font.
    Type3,
    /// Type 0 composite font (CID).
    #[default]
    Type0,
    /// CIDFontType0 descendant font.
    CIDFontType0,
    /// CIDFontType2 descendant font.
    CIDFontType2,
    /// MMType1 font.
    MMType1,
}
/// Font encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FFontEncoding {
    /// Standard encoding.
    Standard,
    /// MacRoman encoding.
    MacRoman,
    /// WinAnsi encoding.
    WinAnsi,
    /// PDFDoc encoding.
    PDFDoc,
    /// MacExpert encoding.
    MacExpert,
    /// Custom encoding with differences.
    Custom(String),
    /// Predefined CMap.
    PredefinedCMap(String),
}
impl Default for FFontEncoding {
    fn default() -> Self {
        FFontEncoding::Standard
    }
}
/// Font descriptor flags.
#[derive(Debug, Clone, Default)]
pub struct FFontDescriptor {
    /// Font name.
    pub font_name: String,
    /// Font family.
    pub font_family: Option<String>,
    /// Font stretch.
    pub font_stretch: Option<String>,
    /// Font weight (100-900).
    pub font_weight: Option<u32>,
    /// Font bounding box [llx, lly, urx, ury].
    pub font_bbox: [f32; 4],
    /// Italic angle.
    pub italic_angle: f32,
    /// Ascent.
    pub ascent: f32,
    /// Descent.
    pub descent: f32,
    /// Leading.
    pub leading: f32,
    /// Cap height.
    pub cap_height: f32,
    /// X height.
    pub x_height: f32,
    /// Stem vertical.
    pub stem_v: f32,
    /// Stem horizontal.
    pub stem_h: f32,
    /// Average width.
    pub avg_width: f32,
    /// Maximum width.
    pub max_width: f32,
    /// Missing width.
    pub missing_width: f32,
    /// Font file embedded reference.
    pub font_file: Option<u32>,
    /// Font file 2 (TrueType) embedded reference.
    pub font_file2: Option<u32>,
    /// Font file 3 (Type 1) embedded reference.
    pub font_file3: Option<u32>,
    /// Character set.
    pub char_set: Option<String>,
    /// Flags (bit field).
    pub flags: u32,
}
/// A font used in the PDF document.
#[derive(Debug, Clone, Default)]
pub struct FFont {
    /// Font name (internal PDF reference name).
    pub name: String,
    /// Font type.
    pub font_type: FFontType,
    /// Base font name.
    pub base_font: Option<String>,
    /// Font encoding.
    pub encoding: FFontEncoding,
    /// Font descriptor.
    pub descriptor: Option<FFontDescriptor>,
    /// ToUnicode CMap reference.
    pub to_unicode: Option<u32>,
    /// First character code in widths array.
    pub first_char: Option<u32>,
    /// Last character code in widths array.
    pub last_char: Option<u32>,
    /// Widths array.
    pub widths: Vec<f32>,
    /// Default width for characters not in widths array.
    pub default_width: Option<f32>,
    /// CIDSystemInfo (for Type 0 fonts).
    pub cid_system_info: Option<String>,
    /// Descendant fonts (for Type 0 fonts).
    pub descendant_fonts: Vec<u32>,
    /// Raw font dictionary.
    pub raw_dict: Option<String>,
}
