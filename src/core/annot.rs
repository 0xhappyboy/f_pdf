//! Annotation structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FAnnotationFlags {
    /// Invisible flag.
    pub invisible: bool,
    /// Hidden flag.
    pub hidden: bool,
    /// Print flag.
    pub print: bool,
    /// No zoom flag.
    pub no_zoom: bool,
    /// No rotate flag.
    pub no_rotate: bool,
    /// No view flag.
    pub no_view: bool,
    /// Read only flag.
    pub read_only: bool,
    /// Locked flag.
    pub locked: bool,
    /// Toggle no view flag.
    pub toggle_no_view: bool,
    /// Locked contents flag.
    pub locked_contents: bool,
}
impl FAnnotationFlags {
    /// Creates flags from a bit field.
    pub fn from_bits(bits: u32) -> Self {
        FAnnotationFlags {
            invisible: bits & 0x0001 != 0,
            hidden: bits & 0x0002 != 0,
            print: bits & 0x0004 != 0,
            no_zoom: bits & 0x0008 != 0,
            no_rotate: bits & 0x0010 != 0,
            no_view: bits & 0x0020 != 0,
            read_only: bits & 0x0040 != 0,
            locked: bits & 0x0080 != 0,
            toggle_no_view: bits & 0x0100 != 0,
            locked_contents: bits & 0x0200 != 0,
        }
    }
    /// Converts flags to a bit field.
    pub fn to_bits(&self) -> u32 {
        let mut bits = 0;
        if self.invisible {
            bits |= 0x0001;
        }
        if self.hidden {
            bits |= 0x0002;
        }
        if self.print {
            bits |= 0x0004;
        }
        if self.no_zoom {
            bits |= 0x0008;
        }
        if self.no_rotate {
            bits |= 0x0010;
        }
        if self.no_view {
            bits |= 0x0020;
        }
        if self.read_only {
            bits |= 0x0040;
        }
        if self.locked {
            bits |= 0x0080;
        }
        if self.toggle_no_view {
            bits |= 0x0100;
        }
        if self.locked_contents {
            bits |= 0x0200;
        }
        bits
    }
}
/// Border style.
#[derive(Debug, Clone, Default)]
pub struct FBorderStyle {
    /// Border width.
    pub width: f32,
    /// Border style (Solid, Dashed, Beveled, Inset, Underline).
    pub style: String,
    /// Dash pattern.
    pub dash_pattern: Vec<f32>,
}
/// An annotation on a page.
#[derive(Debug, Clone, Default)]
pub struct FAnnotation {
    /// Annotation type (Text, Link, FreeText, etc.).
    pub annot_type: String,
    /// Annotation rectangle [x1, y1, x2, y2].
    pub rect: [f32; 4],
    /// Contents (text content of the annotation).
    pub contents: Option<String>,
    /// Author.
    pub author: Option<String>,
    /// Subject.
    pub subject: Option<String>,
    /// Creation date.
    pub creation_date: Option<String>,
    /// Modification date.
    pub modification_date: Option<String>,
    /// Unique name.
    pub name: Option<String>,
    /// Flags.
    pub flags: FAnnotationFlags,
    /// Border style.
    pub border: Option<FBorderStyle>,
    /// Color (RGB).
    pub color: Option<[f32; 3]>,
    /// Destination (for Link annotations).
    pub destination: Option<String>,
    /// Action (for Link annotations).
    pub action: Option<String>,
    /// Appearance stream reference.
    pub appearance: Option<u32>,
    /// Raw annotation dictionary.
    pub raw_dict: Option<String>,
}
