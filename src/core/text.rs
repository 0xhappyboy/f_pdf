//! Text object structures and state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FTextRenderingMode {
    /// Fill text (default).
    #[default]
    Fill,
    /// Stroke text.
    Stroke,
    /// Fill then stroke.
    FillStroke,
    /// Neither fill nor stroke (invisible).
    Invisible,
    /// Fill and add to path for clipping.
    FillClip,
    /// Stroke and add to path for clipping.
    StrokeClip,
    /// Fill, stroke, and add to path for clipping.
    FillStrokeClip,
    /// Add to path for clipping.
    Clip,
}
/// Text matrix for positioning.
#[derive(Debug, Clone, Copy)]
pub struct FTextMatrix {
    /// Matrix values [a, b, c, d, e, f].
    pub values: [f32; 6],
}
impl FTextMatrix {
    /// Creates an identity matrix.
    pub fn identity() -> Self {
        FTextMatrix {
            values: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }
    /// Creates a translation matrix.
    pub fn translation(x: f32, y: f32) -> Self {
        FTextMatrix {
            values: [1.0, 0.0, 0.0, 1.0, x, y],
        }
    }
    /// Multiplies this matrix with another.
    pub fn multiply(&self, other: &FTextMatrix) -> FTextMatrix {
        let a = self.values[0] * other.values[0] + self.values[1] * other.values[2];
        let b = self.values[0] * other.values[1] + self.values[1] * other.values[3];
        let c = self.values[2] * other.values[0] + self.values[3] * other.values[2];
        let d = self.values[2] * other.values[1] + self.values[3] * other.values[3];
        let e =
            self.values[4] * other.values[0] + self.values[5] * other.values[2] + other.values[4];
        let f =
            self.values[4] * other.values[1] + self.values[5] * other.values[3] + other.values[5];
        FTextMatrix {
            values: [a, b, c, d, e, f],
        }
    }
}
impl Default for FTextMatrix {
    fn default() -> Self {
        Self::identity()
    }
}
/// Text state parameters.
#[derive(Debug, Clone)]
pub struct FTextState {
    /// Character spacing in unscaled text units.
    pub char_spacing: f32,
    /// Word spacing in unscaled text units.
    pub word_spacing: f32,
    /// Horizontal scaling percentage (default 100).
    pub horizontal_scaling: f32,
    /// Leading (vertical distance between baselines).
    pub leading: f32,
    /// Text rendering mode.
    pub rendering_mode: FTextRenderingMode,
    /// Text rise (vertical offset).
    pub rise: f32,
    /// Font size.
    pub font_size: f32,
    /// Current text matrix.
    pub matrix: FTextMatrix,
    /// Line matrix (start of current line).
    pub line_matrix: FTextMatrix,
}
impl Default for FTextState {
    fn default() -> Self {
        FTextState {
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            leading: 0.0,
            rendering_mode: FTextRenderingMode::Fill,
            rise: 0.0,
            font_size: 0.0,
            matrix: FTextMatrix::identity(),
            line_matrix: FTextMatrix::identity(),
        }
    }
}
/// A text object in a content stream.
#[derive(Debug, Clone, Default)]
pub struct FTextObject {
    /// Text content.
    pub text: String,
    /// Text state at the time of rendering.
    pub state: FTextState,
    /// Font name reference.
    pub font_name: Option<String>,
    /// Raw content stream snippet.
    pub raw_content: Option<String>,
}
