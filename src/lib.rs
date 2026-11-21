//! PDF document parsing engine.
mod core;
mod error;
mod io;
mod pkg;
mod text;
pub use core::annot::{FAnnotation, FAnnotationFlags, FBorderStyle};
pub use core::docs::{FContent, FPage, FPdf};
pub use core::font::{FFont, FFontDescriptor, FFontEncoding, FFontType};
pub use core::graphics::{
    FGraphicsState, FLineCap, FLineJoin, FPath, FPathSegment, FPathSegmentType,
};
pub use core::image::{FColorSpace, FImage, FImageInterpolation, FImageMask};
pub use core::meta::{FMetadata, FOutline, FOutlineItem};
pub use core::page::{FPageNode, FPageTree};
pub use core::text::{FTextMatrix, FTextObject, FTextState};
pub use error::{FPdfError, FPdfResult};
pub use io::*;
pub use pkg::{FPackage, FPackagePart};
pub use text::*;
