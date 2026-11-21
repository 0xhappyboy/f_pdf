//! Error types for the PDF document engine.
#[derive(Debug)]
pub enum FPdfError {
    /// I/O failure when reading or writing files.
    Io(std::io::Error),
    /// The PDF syntax is malformed or does not match the expected structure.
    Syntax(String),
    /// The requested operation is not valid for the current document state.
    Invalid(String),
    /// A required object could not be found or resolved.
    NotFound(String),
    /// Compression or decompression failed.
    Compression(String),
}
impl std::fmt::Display for FPdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FPdfError::Io(e) => write!(f, "io error: {}", e),
            FPdfError::Syntax(e) => write!(f, "syntax error: {}", e),
            FPdfError::Invalid(e) => write!(f, "invalid operation: {}", e),
            FPdfError::NotFound(e) => write!(f, "not found: {}", e),
            FPdfError::Compression(e) => write!(f, "compression error: {}", e),
        }
    }
}
impl std::error::Error for FPdfError {}
impl From<std::io::Error> for FPdfError {
    fn from(e: std::io::Error) -> Self {
        FPdfError::Io(e)
    }
}
pub type FPdfResult<T> = Result<T, FPdfError>;
