//! Graphics state and path structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FLineCap {
    /// Butt cap (default).
    #[default]
    Butt,
    /// Round cap.
    Round,
    /// Projecting square cap.
    Square,
}
/// Line join style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FLineJoin {
    /// Miter join (default).
    #[default]
    Miter,
    /// Round join.
    Round,
    /// Bevel join.
    Bevel,
}
/// Graphics state parameters.
#[derive(Debug, Clone, Default)]
pub struct FGraphicsState {
    /// Current transformation matrix [a, b, c, d, e, f].
    pub ctm: [f32; 6],
    /// Stroke color (RGB).
    pub stroke_color: [f32; 3],
    /// Fill color (RGB).
    pub fill_color: [f32; 3],
    /// Stroke color space.
    pub stroke_color_space: String,
    /// Fill color space.
    pub fill_color_space: String,
    /// Line width.
    pub line_width: f32,
    /// Line cap style.
    pub line_cap: FLineCap,
    /// Line join style.
    pub line_join: FLineJoin,
    /// Miter limit.
    pub miter_limit: f32,
    /// Dash pattern (dash array and phase).
    pub dash_pattern: (Vec<f32>, f32),
    /// Rendering intent.
    pub rendering_intent: String,
    /// Stroke adjustment flag.
    pub stroke_adjustment: bool,
    /// Overprint stroke flag.
    pub overprint_stroke: bool,
    /// Overprint fill flag.
    pub overprint_fill: bool,
    /// Overprint mode.
    pub overprint_mode: u32,
    /// Black generation function.
    pub black_generation: Option<String>,
    /// Undercolor removal function.
    pub undercolor_removal: Option<String>,
    /// Transfer function.
    pub transfer: Option<String>,
    /// Halftone dictionary.
    pub halftone: Option<String>,
    /// Flatness tolerance.
    pub flatness: f32,
    /// Smoothness tolerance.
    pub smoothness: f32,
}
impl FGraphicsState {
    /// Creates a default graphics state.
    pub fn new() -> Self {
        FGraphicsState {
            ctm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            stroke_color: [0.0, 0.0, 0.0],
            fill_color: [0.0, 0.0, 0.0],
            stroke_color_space: "DeviceGray".to_string(),
            fill_color_space: "DeviceGray".to_string(),
            line_width: 1.0,
            line_cap: FLineCap::Butt,
            line_join: FLineJoin::Miter,
            miter_limit: 10.0,
            dash_pattern: (Vec::new(), 0.0),
            rendering_intent: "RelativeColorimetric".to_string(),
            stroke_adjustment: false,
            overprint_stroke: false,
            overprint_fill: false,
            overprint_mode: 0,
            black_generation: None,
            undercolor_removal: None,
            transfer: None,
            halftone: None,
            flatness: 1.0,
            smoothness: 0.0,
        }
    }
}
/// Path segment type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FPathSegmentType {
    /// Move to point.
    #[default]
    MoveTo,
    /// Line to point.
    LineTo,
    /// Cubic Bezier curve to point.
    CurveTo,
    /// Close path.
    Close,
}
/// A segment in a path.
#[derive(Debug, Clone, Default)]
pub struct FPathSegment {
    /// Segment type.
    pub segment_type: FPathSegmentType,
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
    /// Control point 1 X (for curves).
    pub x1: f32,
    /// Control point 1 Y (for curves).
    pub y1: f32,
    /// Control point 2 X (for curves).
    pub x2: f32,
    /// Control point 2 Y (for curves).
    pub y2: f32,
}
/// A path in a content stream.
#[derive(Debug, Clone, Default)]
pub struct FPath {
    /// Segments composing the path.
    pub segments: Vec<FPathSegment>,
    /// Current point.
    pub current_point: (f32, f32),
    /// Starting point of subpath.
    pub subpath_start: (f32, f32),
}
impl FPath {
    /// Creates an empty path.
    pub fn new() -> Self {
        FPath {
            segments: Vec::new(),
            current_point: (0.0, 0.0),
            subpath_start: (0.0, 0.0),
        }
    }
    /// Moves to a new point.
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.current_point = (x, y);
        self.subpath_start = (x, y);
        self.segments.push(FPathSegment {
            segment_type: FPathSegmentType::MoveTo,
            x,
            y,
            ..Default::default()
        });
    }
    /// Draws a line to a point.
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.current_point = (x, y);
        self.segments.push(FPathSegment {
            segment_type: FPathSegmentType::LineTo,
            x,
            y,
            ..Default::default()
        });
    }
    /// Draws a cubic Bezier curve.
    pub fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.current_point = (x, y);
        self.segments.push(FPathSegment {
            segment_type: FPathSegmentType::CurveTo,
            x,
            y,
            x1,
            y1,
            x2,
            y2,
        });
    }
    /// Closes the current subpath.
    pub fn close(&mut self) {
        self.current_point = self.subpath_start;
        self.segments.push(FPathSegment {
            segment_type: FPathSegmentType::Close,
            ..Default::default()
        });
    }
}
