//! Document parsing logic.
use crate::core::annot::{FAnnotation, FAnnotationFlags, FBorderStyle};
use crate::core::docs::{FContent, FPage, FPdf};
use crate::core::font::{FFont, FFontDescriptor, FFontEncoding, FFontType};
use crate::core::image::{FColorSpace, FImage};
use crate::core::meta::{FMetadata, FOutline, FOutlineItem};
use crate::core::page::{FPageNode, FPageTree};
use crate::{
    FPackage, FPackagePart, FPdfError, FPdfResult, decode_hex_string, decode_pdf_string,
    extract_delimited_text, extract_dict_value, extract_reference, pattern_exists,
};
use std::collections::HashMap;
use std::io::Read;
/// Parses a complete PDF document from raw bytes.
pub(crate) fn parse_document(data: &[u8]) -> crate::FPdfResult<FPdf> {
    if data.len() < 5 || !data.starts_with(b"%PDF-") {
        return Err(FPdfError::Syntax("invalid PDF header".to_string()));
    }
    let mut package = FPackage::default();
    package.main_part = "document.pdf".to_string();
    let mut header_version = String::new();
    if let Ok(header) = std::str::from_utf8(&data[..std::cmp::min(20, data.len())]) {
        if let Some(version) = header.strip_prefix("%PDF-") {
            header_version = version.trim().to_string();
        }
    }
    let (xref_table, trailer) = parse_xref_and_trailer(data)?;
    let mut objects: HashMap<u32, (u32, Vec<u8>)> = HashMap::new();
    for (&obj_num, &(offset, r#gen)) in &xref_table {
        if let Some(obj_data) = extract_object_at_offset(data, offset as usize) {
            objects.insert(obj_num, (r#gen, obj_data));
        }
    }
    let mut root_ref: Option<u32> = None;
    if let Some(root) = extract_dict_value(&trailer, "Root") {
        if let Some((num, _)) = extract_reference(&root) {
            root_ref = Some(num);
        }
    }
    let mut doc = FPdf::default();
    doc.version = header_version.clone();
    if let Some(root) = root_ref {
        if let Some((_, root_data)) = objects.get(&root) {
            if let Ok(root_str) = std::str::from_utf8(root_data) {
                doc = parse_catalog(root_str, objects.clone(), &trailer)?;
                if doc.version.is_empty() {
                    doc.version = header_version;
                }
            }
        }
    }
    let mut parts = Vec::new();
    for (obj_num, (r#gen, obj_data)) in objects {
        parts.push(FPackagePart {
            object_number: obj_num,
            generation: r#gen,
            data: obj_data,
        });
    }
    package.parts = parts;
    doc.package = package;
    doc.dirty = false;
    Ok(doc)
}
/// Parses the xref table and trailer.
fn parse_xref_and_trailer(data: &[u8]) -> FPdfResult<(HashMap<u32, (u32, u32)>, String)> {
    let mut xref_table = HashMap::new();
    let mut trailer = String::new();
    let data_str = String::from_utf8_lossy(data);
    if let Some(startxref_pos) = data_str.rfind("startxref") {
        let after = &data_str[startxref_pos + 9..];
        for line in after.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(xref_offset) = trimmed.parse::<usize>() {
                if let Some(xref_data) = data.get(xref_offset..) {
                    if let Ok(xref_str) = std::str::from_utf8(xref_data) {
                        let mut lines = xref_str.lines();
                        if let Some(first_line) = lines.next() {
                            if first_line.trim() == "xref" {
                                while let Some(line) = lines.next() {
                                    let line = line.trim();
                                    if line.is_empty() {
                                        continue;
                                    }
                                    if line == "trailer" {
                                        let mut trailer_lines = Vec::new();
                                        for trailer_line in lines.by_ref() {
                                            let t = trailer_line.trim();
                                            if t == "startxref" || t.starts_with("%%EOF") {
                                                break;
                                            }
                                            trailer_lines.push(trailer_line);
                                        }
                                        trailer = trailer_lines.join("\n");
                                        break;
                                    }
                                    let parts: Vec<&str> = line.split_whitespace().collect();
                                    if parts.len() == 2 {
                                        if let (Ok(start), Ok(count)) =
                                            (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                                        {
                                            for i in 0..count {
                                                if let Some(entry_line) = lines.next() {
                                                    let entry = entry_line.trim();
                                                    let entry_parts: Vec<&str> =
                                                        entry.split_whitespace().collect();
                                                    if entry_parts.len() >= 3 {
                                                        if let (Ok(offset), Ok(r#gen)) = (
                                                            entry_parts[0].parse::<u32>(),
                                                            entry_parts[1].parse::<u32>(),
                                                        ) {
                                                            xref_table
                                                                .insert(start + i, (offset, r#gen));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                break;
            }
        }
    }
    if xref_table.is_empty() {
        let mut pos = 0;
        while pos < data.len() {
            if let Some(rel_pos) = find_pattern(&data[pos..], b" obj") {
                let abs_pos = pos + rel_pos;
                let start = find_object_start(data, abs_pos);
                if let Some(obj_start) = start {
                    let header = String::from_utf8_lossy(&data[obj_start..abs_pos]);
                    let parts: Vec<&str> = header.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let (Ok(obj_num), Ok(r#gen)) =
                            (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                        {
                            xref_table.insert(obj_num, (obj_start as u32, r#gen));
                        }
                    }
                }
                pos = abs_pos + 4;
            } else {
                break;
            }
        }
    }
    if trailer.is_empty() {
        if let Some(pos) = data_str.rfind("trailer") {
            let after = &data_str[pos + 7..];
            let mut trailer_lines = Vec::new();
            for trailer_line in after.lines() {
                let t = trailer_line.trim();
                if t == "startxref" || t.starts_with("%%EOF") {
                    break;
                }
                trailer_lines.push(trailer_line);
            }
            trailer = trailer_lines.join("\n");
        }
    }
    Ok((xref_table, trailer))
}
/// Finds a byte pattern in data.
fn find_pattern(data: &[u8], pattern: &[u8]) -> Option<usize> {
    if pattern.is_empty() || data.len() < pattern.len() {
        return None;
    }
    for i in 0..=(data.len() - pattern.len()) {
        if &data[i..i + pattern.len()] == pattern {
            return Some(i);
        }
    }
    None
}
/// Finds the start of an object (going backwards to find the beginning of the line).
fn find_object_start(data: &[u8], pos: usize) -> Option<usize> {
    let mut i = pos;
    while i > 0 {
        if data[i] == b'\n' || data[i] == b'\r' {
            return Some(i + 1);
        }
        i -= 1;
    }
    Some(0)
}
/// Extracts an object at a given offset.
fn extract_object_at_offset(data: &[u8], offset: usize) -> Option<Vec<u8>> {
    if offset >= data.len() {
        return None;
    }
    let mut end = offset;
    while end < data.len() {
        if end + 6 <= data.len() && &data[end..end + 6] == b"endobj" {
            break;
        }
        end += 1;
    }
    if end + 6 <= data.len() {
        Some(data[offset..end + 6].to_vec())
    } else {
        Some(data[offset..].to_vec())
    }
}
/// Parses the document catalog.
fn parse_catalog(
    catalog: &str,
    objects: HashMap<u32, (u32, Vec<u8>)>,
    trailer: &str,
) -> FPdfResult<FPdf> {
    let mut doc = FPdf::default();
    let info_source = if pattern_exists(catalog, "/Info") {
        catalog.to_string()
    } else {
        trailer.to_string()
    };
    if let Some(info_ref) = extract_dict_value(&info_source, "Info") {
        if let Some((num, _)) = extract_reference(&info_ref) {
            if let Some((_, info_data)) = objects.get(&num) {
                if let Ok(info_str) = std::str::from_utf8(info_data) {
                    doc.metadata = parse_metadata(info_str);
                }
            }
        }
    }
    if let Some(pages_ref) = extract_dict_value(catalog, "Pages") {
        if let Some((num, _)) = extract_reference(&pages_ref) {
            doc.page_tree = parse_page_tree(num, &objects)?;
        }
    }
    if let Some(outline_ref) = extract_dict_value(catalog, "Outlines") {
        if let Some((num, _)) = extract_reference(&outline_ref) {
            if let Some((_, outline_data)) = objects.get(&num) {
                if let Ok(outline_str) = std::str::from_utf8(outline_data) {
                    doc.outline = Some(parse_outline(outline_str, &objects));
                }
            }
        }
    }
    let page_numbers = doc.page_tree.get_page_numbers();
    for (i, &page_num) in page_numbers.iter().enumerate() {
        if let Some((_, page_data)) = objects.get(&page_num) {
            if let Ok(page_str) = std::str::from_utf8(page_data) {
                let page = parse_page(page_str, i as u32 + 1, &objects)?;
                doc.pages.push(page);
            }
        }
    }
    if doc.pages.is_empty() && !objects.is_empty() {
        let mut page_entries: Vec<(u32, Vec<u8>)> = objects
            .iter()
            .filter_map(|(&num, (_, data))| {
                let s = std::str::from_utf8(data).ok()?;
                if pattern_exists(s, "/Type /Page") && !pattern_exists(s, "/Type /Pages") {
                    Some((num, data.clone()))
                } else {
                    None
                }
            })
            .collect();
        page_entries.sort_by_key(|(num, _)| *num);
        for (i, (_, page_data)) in page_entries.iter().enumerate() {
            if let Ok(page_str) = std::str::from_utf8(page_data) {
                let page = parse_page(page_str, i as u32 + 1, &objects)?;
                doc.pages.push(page);
            }
        }
    }
    doc.fonts = parse_fonts(&objects);
    Ok(doc)
}
/// Parses document metadata.
fn parse_metadata(info: &str) -> FMetadata {
    let mut metadata = FMetadata::default();
    if let Some(title) = extract_delimited_text(info, "/Title (", ")") {
        metadata.title = Some(decode_pdf_string(&title));
    }
    if let Some(author) = extract_delimited_text(info, "/Author (", ")") {
        metadata.author = Some(decode_pdf_string(&author));
    }
    if let Some(subject) = extract_delimited_text(info, "/Subject (", ")") {
        metadata.subject = Some(decode_pdf_string(&subject));
    }
    if let Some(keywords) = extract_delimited_text(info, "/Keywords (", ")") {
        metadata.keywords = Some(decode_pdf_string(&keywords));
    }
    if let Some(creator) = extract_delimited_text(info, "/Creator (", ")") {
        metadata.creator = Some(decode_pdf_string(&creator));
    }
    if let Some(producer) = extract_delimited_text(info, "/Producer (", ")") {
        metadata.producer = Some(decode_pdf_string(&producer));
    }
    if let Some(creation_date) = extract_delimited_text(info, "/CreationDate (", ")") {
        metadata.creation_date = Some(creation_date);
    }
    if let Some(mod_date) = extract_delimited_text(info, "/ModDate (", ")") {
        metadata.modification_date = Some(mod_date);
    }
    if let Some(trapped) = extract_dict_value(info, "Trapped") {
        metadata.trapped = Some(trapped);
    }
    metadata
}
/// Parses the page tree.
fn parse_page_tree(root_num: u32, objects: &HashMap<u32, (u32, Vec<u8>)>) -> FPdfResult<FPageTree> {
    let mut tree = FPageTree::new();
    tree.root = Some(root_num);
    let mut queue = vec![root_num];
    let mut visited = std::collections::HashSet::new();
    while let Some(node_num) = queue.pop() {
        if visited.contains(&node_num) {
            continue;
        }
        visited.insert(node_num);
        if let Some((_, node_data)) = objects.get(&node_num) {
            if let Ok(node_str) = std::str::from_utf8(node_data) {
                let mut node = FPageNode::default();
                if pattern_exists(node_str, "/Type /Pages") {
                    node.node_type = "Pages".to_string();
                } else if pattern_exists(node_str, "/Type /Page") {
                    node.node_type = "Page".to_string();
                }
                if let Some(parent) = extract_dict_value(node_str, "Parent") {
                    if let Some((num, _)) = extract_reference(&parent) {
                        node.parent = Some(num);
                    }
                }
                if let Some(kids) = extract_dict_value(node_str, "Kids") {
                    let kids_str = kids.trim_start_matches('[').trim_end_matches(']');
                    for kid in kids_str.split(|c: char| c == ',' || c.is_whitespace()) {
                        let kid = kid.trim();
                        if kid.is_empty() {
                            continue;
                        }
                        if let Some((num, _)) = extract_reference(kid) {
                            node.kids.push(num);
                            queue.push(num);
                        }
                    }
                }
                if let Some(count) = extract_dict_value(node_str, "Count") {
                    node.count = count.parse().ok();
                }
                if let Some(media_box) = extract_dict_value(node_str, "MediaBox") {
                    node.media_box = Some(parse_rect(&media_box));
                }
                if let Some(crop_box) = extract_dict_value(node_str, "CropBox") {
                    node.crop_box = Some(parse_rect(&crop_box));
                }
                if let Some(rotation) = extract_dict_value(node_str, "Rotate") {
                    node.rotation = rotation.parse().ok();
                }
                tree.add_node(node_num, node);
            }
        }
    }
    Ok(tree)
}
/// Parses a rectangle [x1, y1, x2, y2].
fn parse_rect(s: &str) -> [f32; 4] {
    let cleaned = s.trim_start_matches('[').trim_end_matches(']');
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    if parts.len() >= 4 {
        [
            parts[0].parse().unwrap_or(0.0),
            parts[1].parse().unwrap_or(0.0),
            parts[2].parse().unwrap_or(0.0),
            parts[3].parse().unwrap_or(0.0),
        ]
    } else {
        [0.0, 0.0, 0.0, 0.0]
    }
}
/// Parses a page.
fn parse_page(
    page_str: &str,
    page_number: u32,
    objects: &HashMap<u32, (u32, Vec<u8>)>,
) -> FPdfResult<FPage> {
    let mut page = FPage {
        page_number,
        ..Default::default()
    };
    if let Some(media_box) = extract_dict_value(page_str, "MediaBox") {
        page.media_box = parse_rect(&media_box);
    }
    if let Some(crop_box) = extract_dict_value(page_str, "CropBox") {
        page.crop_box = Some(parse_rect(&crop_box));
    }
    if let Some(rotation) = extract_dict_value(page_str, "Rotate") {
        page.rotation = rotation.parse().unwrap_or(0);
    }
    if let Some(resources) = extract_dict_value(page_str, "Resources") {
        page.resources = Some(resources);
    }
    // Parse contents
    if let Some(contents) = extract_dict_value(page_str, "Contents") {
        let contents_str = contents.trim();
        if contents_str.starts_with('[') {
            // Array of content streams
            let arr = contents_str.trim_start_matches('[').trim_end_matches(']');
            for item in arr.split(|c: char| c == ',' || c.is_whitespace()) {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }
                if let Some((num, _)) = extract_reference(item) {
                    if let Some((_, content_data)) = objects.get(&num) {
                        page.contents.push(FContent {
                            data: content_data.clone(),
                            compressed: false,
                            filters: Vec::new(),
                        });
                    }
                }
            }
        } else if let Some((num, _)) = extract_reference(contents_str) {
            if let Some((_, content_data)) = objects.get(&num) {
                page.contents.push(FContent {
                    data: content_data.clone(),
                    compressed: false,
                    filters: Vec::new(),
                });
            }
        }
    }
    // Parse annotations
    if let Some(annots) = extract_dict_value(page_str, "Annots") {
        let annots_str = annots.trim_start_matches('[').trim_end_matches(']');
        for annot_ref in annots_str.split(|c: char| c == ',' || c.is_whitespace()) {
            let annot_ref = annot_ref.trim();
            if annot_ref.is_empty() {
                continue;
            }
            if let Some((num, _)) = extract_reference(annot_ref) {
                if let Some((_, annot_data)) = objects.get(&num) {
                    if let Ok(annot_str) = std::str::from_utf8(annot_data) {
                        page.annotations.push(parse_annotation(annot_str));
                    }
                }
            }
        }
    }
    page.raw_dict = Some(page_str.to_string());
    Ok(page)
}
/// Parses an annotation.
fn parse_annotation(annot_str: &str) -> FAnnotation {
    let mut annot = FAnnotation::default();
    if let Some(annot_type) = extract_dict_value(annot_str, "Subtype") {
        annot.annot_type = annot_type.trim_start_matches('/').to_string();
    }
    if let Some(rect) = extract_dict_value(annot_str, "Rect") {
        annot.rect = parse_rect(&rect);
    }
    if let Some(contents) = extract_delimited_text(annot_str, "/Contents (", ")") {
        annot.contents = Some(decode_pdf_string(&contents));
    }
    if let Some(author) = extract_delimited_text(annot_str, "/T (", ")") {
        annot.author = Some(decode_pdf_string(&author));
    }
    if let Some(subject) = extract_delimited_text(annot_str, "/Subj (", ")") {
        annot.subject = Some(decode_pdf_string(&subject));
    }
    if let Some(name) = extract_delimited_text(annot_str, "/NM (", ")") {
        annot.name = Some(decode_pdf_string(&name));
    }
    if let Some(creation_date) = extract_delimited_text(annot_str, "/CreationDate (", ")") {
        annot.creation_date = Some(creation_date);
    }
    if let Some(mod_date) = extract_delimited_text(annot_str, "/M (", ")") {
        annot.modification_date = Some(mod_date);
    }
    if let Some(flags) = extract_dict_value(annot_str, "F") {
        if let Ok(bits) = flags.parse::<u32>() {
            annot.flags = FAnnotationFlags::from_bits(bits);
        }
    }
    if let Some(border) = extract_dict_value(annot_str, "Border") {
        annot.border = Some(parse_border(&border));
    }
    if let Some(color) = extract_dict_value(annot_str, "C") {
        annot.color = Some(parse_color(&color));
    }
    if let Some(dest) = extract_dict_value(annot_str, "Dest") {
        annot.destination = Some(dest);
    }
    if let Some(action) = extract_dict_value(annot_str, "A") {
        annot.action = Some(action);
    }
    annot.raw_dict = Some(annot_str.to_string());
    annot
}
/// Parses border style.
fn parse_border(s: &str) -> FBorderStyle {
    let mut border = FBorderStyle::default();
    let cleaned = s.trim_start_matches('[').trim_end_matches(']');
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    if parts.len() >= 2 {
        border.width = parts[1].parse().unwrap_or(1.0);
    }
    if parts.len() >= 3 {
        border.style = parts[2].trim_start_matches('/').to_string();
    }
    border
}
/// Parses color [r, g, b].
fn parse_color(s: &str) -> [f32; 3] {
    let cleaned = s.trim_start_matches('[').trim_end_matches(']');
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    if parts.len() >= 3 {
        [
            parts[0].parse().unwrap_or(0.0),
            parts[1].parse().unwrap_or(0.0),
            parts[2].parse().unwrap_or(0.0),
        ]
    } else {
        [0.0, 0.0, 0.0]
    }
}
/// Parses fonts from objects.
fn parse_fonts(objects: &HashMap<u32, (u32, Vec<u8>)>) -> Vec<FFont> {
    let mut fonts = Vec::new();
    for (obj_num, (_, obj_data)) in objects {
        if let Ok(obj_str) = std::str::from_utf8(obj_data) {
            if pattern_exists(obj_str, "/Type /Font") {
                if let Some(font) = parse_font(obj_str) {
                    fonts.push(font);
                }
            }
        }
    }
    fonts
}
/// Parses a font dictionary.
fn parse_font(font_str: &str) -> Option<FFont> {
    let mut font = FFont::default();
    if let Some(subtype) = extract_dict_value(font_str, "Subtype") {
        font.font_type = match subtype.trim_start_matches('/') {
            "Type1" => FFontType::Type1,
            "TrueType" => FFontType::TrueType,
            "Type3" => FFontType::Type3,
            "Type0" => FFontType::Type0,
            "CIDFontType0" => FFontType::CIDFontType0,
            "CIDFontType2" => FFontType::CIDFontType2,
            "MMType1" => FFontType::MMType1,
            _ => FFontType::Type1,
        };
    }
    if let Some(base_font) = extract_dict_value(font_str, "BaseFont") {
        font.base_font = Some(base_font.trim_start_matches('/').to_string());
    }
    if let Some(name) = extract_dict_value(font_str, "Name") {
        font.name = name.trim_start_matches('/').to_string();
    }
    if let Some(encoding) = extract_dict_value(font_str, "Encoding") {
        font.encoding = match encoding.trim_start_matches('/') {
            "StandardEncoding" => FFontEncoding::Standard,
            "MacRomanEncoding" => FFontEncoding::MacRoman,
            "WinAnsiEncoding" => FFontEncoding::WinAnsi,
            "PDFDocEncoding" => FFontEncoding::PDFDoc,
            "MacExpertEncoding" => FFontEncoding::MacExpert,
            e if e.starts_with("Identity-") => FFontEncoding::PredefinedCMap(e.to_string()),
            e => FFontEncoding::Custom(e.to_string()),
        };
    }
    if let Some(first_char) = extract_dict_value(font_str, "FirstChar") {
        font.first_char = first_char.parse().ok();
    }
    if let Some(last_char) = extract_dict_value(font_str, "LastChar") {
        font.last_char = last_char.parse().ok();
    }
    if let Some(widths) = extract_dict_value(font_str, "Widths") {
        let widths_str = widths.trim_start_matches('[').trim_end_matches(']');
        font.widths = widths_str
            .split_whitespace()
            .filter_map(|w| w.parse().ok())
            .collect();
    }
    if let Some(default_width) = extract_dict_value(font_str, "MissingWidth") {
        font.default_width = default_width.parse().ok();
    }
    if let Some(to_unicode) = extract_dict_value(font_str, "ToUnicode") {
        if let Some((num, _)) = extract_reference(&to_unicode) {
            font.to_unicode = Some(num);
        }
    }
    font.raw_dict = Some(font_str.to_string());
    Some(font)
}
/// Parses the document outline.
fn parse_outline(outline_str: &str, objects: &HashMap<u32, (u32, Vec<u8>)>) -> FOutline {
    let mut outline = FOutline::default();
    if let Some(first) = extract_dict_value(outline_str, "First") {
        if let Some((num, _)) = extract_reference(&first) {
            let mut current = Some(num);
            while let Some(item_num) = current {
                if let Some(item) = parse_outline_item(item_num, objects) {
                    current = item.next;
                    outline.items.push(item);
                } else {
                    break;
                }
            }
        }
    }
    outline.raw_dict = Some(outline_str.to_string());
    outline
}
/// Parses an outline item.
fn parse_outline_item(
    obj_num: u32,
    objects: &HashMap<u32, (u32, Vec<u8>)>,
) -> Option<FOutlineItem> {
    let (_, obj_data) = objects.get(&obj_num)?;
    let obj_str = std::str::from_utf8(obj_data).ok()?;
    let mut item = FOutlineItem::default();
    if let Some(title) = extract_delimited_text(obj_str, "/Title (", ")") {
        item.title = decode_pdf_string(&title);
    }
    if let Some(dest) = extract_dict_value(obj_str, "Dest") {
        item.dest = Some(dest);
    }
    if let Some(action) = extract_dict_value(obj_str, "A") {
        item.action = Some(action);
    }
    if let Some(parent) = extract_dict_value(obj_str, "Parent") {
        if let Some((num, _)) = extract_reference(&parent) {
            item.parent = Some(num);
        }
    }
    if let Some(prev) = extract_dict_value(obj_str, "Prev") {
        if let Some((num, _)) = extract_reference(&prev) {
            item.prev = Some(num);
        }
    }
    if let Some(next) = extract_dict_value(obj_str, "Next") {
        if let Some((num, _)) = extract_reference(&next) {
            item.next = Some(num);
        }
    }
    if let Some(first) = extract_dict_value(obj_str, "First") {
        if let Some((num, _)) = extract_reference(&first) {
            item.first = Some(num);
            // Parse children recursively
            if let Some(child) = parse_outline_item(num, objects) {
                item.children.push(child);
            }
        }
    }
    if let Some(last) = extract_dict_value(obj_str, "Last") {
        if let Some((num, _)) = extract_reference(&last) {
            item.last = Some(num);
        }
    }
    if let Some(count) = extract_dict_value(obj_str, "Count") {
        item.count = count.parse().ok();
    }
    if let Some(color) = extract_dict_value(obj_str, "C") {
        item.color = Some(parse_color(&color));
    }
    if let Some(flags) = extract_dict_value(obj_str, "F") {
        item.font_flags = flags.parse().ok();
    }
    Some(item)
}
