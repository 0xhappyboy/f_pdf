//! Root document structure.
use super::font::FFont;
use super::meta::FMetadata;
use super::page::FPageTree;
use crate::FPackage;
/// Content stream of a page.
#[derive(Debug, Clone, Default)]
pub struct FContent {
    /// Raw content stream data.
    pub data: Vec<u8>,
    /// Whether the content is compressed with FlateDecode.
    pub compressed: bool,
    /// Filters applied to the content stream.
    pub filters: Vec<String>,
}
/// A single page in the PDF document.
#[derive(Debug, Clone, Default)]
pub struct FPage {
    /// Page number (1-based).
    pub page_number: u32,
    /// Media box (page dimensions) [x1, y1, x2, y2].
    pub media_box: [f32; 4],
    /// Crop box (visible area) [x1, y1, x2, y2].
    pub crop_box: Option<[f32; 4]>,
    /// Rotation angle in degrees (0, 90, 180, 270).
    pub rotation: u32,
    /// Content streams of the page.
    pub contents: Vec<FContent>,
    /// Resources dictionary (fonts, images, etc.).
    pub resources: Option<String>,
    /// Annotations on this page.
    pub annotations: Vec<super::annot::FAnnotation>,
    /// Raw page dictionary content.
    pub raw_dict: Option<String>,
}
/// Root of the PDF document data model.
#[derive(Debug, Clone, Default)]
pub struct FPdf {
    /// Package-level objects that are not modeled explicitly.
    pub package: FPackage,
    /// Document metadata.
    pub metadata: FMetadata,
    /// Page tree structure.
    pub page_tree: FPageTree,
    /// Pages in the document.
    pub pages: Vec<FPage>,
    /// Fonts used in the document.
    pub fonts: Vec<FFont>,
    /// Document outline (bookmarks).
    pub outline: Option<super::meta::FOutline>,
    /// PDF version string (e.g., "1.7").
    pub version: String,
    /// Whether the document has been modified since it was loaded.
    pub dirty: bool,
}
impl FPdf {
    /// Creates an empty in-memory PDF document.
    pub fn new_empty() -> Self {
        FPdf {
            version: "1.7".to_string(),
            ..Default::default()
        }
    }
    /// Loads a PDF document from a file on disk.
    pub fn with_path<P: AsRef<std::path::Path>>(path: P) -> crate::FPdfResult<Self> {
        let data = std::fs::read(path)?;
        Self::with_bytes(&data)
    }
    /// Parses a PDF document from its raw bytes.
    pub fn with_bytes(data: &[u8]) -> crate::FPdfResult<Self> {
        crate::io::reader::parse_document(data)
    }
    /// Serializes the document back into PDF bytes.
    pub fn to_bytes(&self) -> crate::FPdfResult<Vec<u8>> {
        crate::io::writer::serialize_document(self)
    }
    /// Marks the document as modified.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    /// Clears the modified flag.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
    /// Returns the plain text content of the document.
    pub fn plain_text(&self) -> String {
        let mut text = String::new();
        for page in &self.pages {
            for content in &page.contents {
                if let Ok(content_str) = std::str::from_utf8(&content.data) {
                    // Extract text from content stream
                    text.push_str(&extract_text_from_content(content_str));
                }
            }
            text.push('\n');
        }
        text
    }
    /// Returns the number of pages in the document.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
    /// Returns a reference to a page by number (1-based).
    pub fn get_page(&self, page_number: u32) -> Option<&FPage> {
        self.pages.iter().find(|p| p.page_number == page_number)
    }
    /// Returns a mutable reference to a page by number (1-based).
    pub fn get_page_mut(&mut self, page_number: u32) -> Option<&mut FPage> {
        self.pages.iter_mut().find(|p| p.page_number == page_number)
    }
}
/// Extracts text from a content stream.
fn extract_text_from_content(content: &str) -> String {
    let mut text = String::new();
    let mut in_text_block = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "BT" {
            in_text_block = true;
        } else if trimmed == "ET" {
            in_text_block = false;
        } else if in_text_block {
            // Look for Tj (show text) operator
            if let Some(start) = trimmed.find('(') {
                if let Some(end) = trimmed.rfind(')') {
                    if end > start {
                        let s = &trimmed[start + 1..end];
                        text.push_str(&crate::text::decode_pdf_string(s));
                    }
                }
            }
            // Look for TJ (show text with spacing) operator
            if trimmed.contains('[') && trimmed.contains(']') {
                if let Some(arr_start) = trimmed.find('[') {
                    if let Some(arr_end) = trimmed.find(']') {
                        let arr = &trimmed[arr_start + 1..arr_end];
                        // Extract strings from array
                        let mut pos = 0;
                        while let Some(rel_start) = arr[pos..].find('(') {
                            let abs_start = pos + rel_start;
                            if let Some(rel_end) = arr[abs_start..].find(')') {
                                let abs_end = abs_start + rel_end;
                                let s = &arr[abs_start + 1..abs_end];
                                text.push_str(&crate::text::decode_pdf_string(s));
                                pos = abs_end + 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    text
}
