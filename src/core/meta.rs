//! Document metadata and outline structures.
#[derive(Debug, Clone, Default)]
pub struct FMetadata {
    /// Document title.
    pub title: Option<String>,
    /// Author.
    pub author: Option<String>,
    /// Subject.
    pub subject: Option<String>,
    /// Keywords.
    pub keywords: Option<String>,
    /// Creator application.
    pub creator: Option<String>,
    /// Producer application.
    pub producer: Option<String>,
    /// Creation date.
    pub creation_date: Option<String>,
    /// Modification date.
    pub modification_date: Option<String>,
    /// Trapped status.
    pub trapped: Option<String>,
    /// Custom metadata entries.
    pub custom: std::collections::HashMap<String, String>,
}
/// An item in the document outline (bookmark).
#[derive(Debug, Clone, Default)]
pub struct FOutlineItem {
    /// Title of the outline item.
    pub title: String,
    /// Destination page reference.
    pub dest: Option<String>,
    /// Action to perform.
    pub action: Option<String>,
    /// Children (sub-items).
    pub children: Vec<FOutlineItem>,
    /// Parent item reference.
    pub parent: Option<u32>,
    /// Previous sibling reference.
    pub prev: Option<u32>,
    /// Next sibling reference.
    pub next: Option<u32>,
    /// First child reference.
    pub first: Option<u32>,
    /// Last child reference.
    pub last: Option<u32>,
    /// Count of open descendants.
    pub count: Option<i32>,
    /// Text color.
    pub color: Option<[f32; 3]>,
    /// Font flags (bit 1 = italic, bit 2 = bold).
    pub font_flags: Option<u32>,
}
/// The document outline (bookmarks).
#[derive(Debug, Clone, Default)]
pub struct FOutline {
    /// Root outline items.
    pub items: Vec<FOutlineItem>,
    /// Raw outline dictionary.
    pub raw_dict: Option<String>,
}
