//! Document serialization logic.
use crate::core::annot::{FAnnotation, FAnnotationFlags, FBorderStyle};
use crate::core::docs::{FContent, FPage, FPdf};
use crate::core::font::{FFont, FFontEncoding, FFontType};
use crate::core::image::{FColorSpace, FImage};
use crate::core::meta::{FMetadata, FOutline, FOutlineItem};
use crate::core::page::FPageTree;
use crate::{FPdfResult, encode_hex_string, encode_pdf_string};
use std::io::Write;
/// PDF header.
const PDF_HEADER: &str = "%PDF-1.7\n";
/// EOF marker.
const PDF_EOF: &str = "%%EOF\n";
/// Serializes a complete PDF document to bytes.
pub(crate) fn serialize_document(doc: &FPdf) -> FPdfResult<Vec<u8>> {
    let mut output = Vec::new();
    let mut offsets: Vec<(u32, u32, usize)> = Vec::new();
    let mut next_obj_num: u32 = 1;
    output.write_all(PDF_HEADER.as_bytes())?;
    output.write_all(b"%\xE2\xE3\xCF\xD3\n")?;
    let catalog_num = next_obj_num;
    next_obj_num += 1;
    let pages_num = next_obj_num;
    next_obj_num += 1;
    let info_num = next_obj_num;
    next_obj_num += 1;
    let mut font_obj_nums = Vec::new();
    for font in &doc.fonts {
        let font_num = next_obj_num;
        next_obj_num += 1;
        font_obj_nums.push(font_num);
        offsets.push((font_num, 0, output.len()));
        write_font_object(&mut output, font_num, font)?;
    }
    let mut image_obj_nums: Vec<u32> = Vec::new();
    let images = collect_images(doc);
    for image in &images {
        let image_num = next_obj_num;
        next_obj_num += 1;
        image_obj_nums.push(image_num);
        offsets.push((image_num, 0, output.len()));
        write_image_object(&mut output, image_num, image)?;
    }
    let mut page_obj_nums = Vec::new();
    for (page_index, page) in doc.pages.iter().enumerate() {
        let page_num = next_obj_num;
        next_obj_num += 1;
        page_obj_nums.push(page_num);
        offsets.push((page_num, 0, output.len()));
        write_page_object(
            &mut output,
            page_num,
            page,
            pages_num,
            &font_obj_nums,
            &image_obj_nums,
            page_index,
            &mut offsets,
            &mut next_obj_num,
        )?;
    }
    offsets.push((pages_num, 0, output.len()));
    write_page_tree_object(&mut output, pages_num, &page_obj_nums)?;
    offsets.push((info_num, 0, output.len()));
    write_info_object(&mut output, info_num, &doc.metadata)?;
    let outline_num = if doc.outline.is_some() {
        let n = next_obj_num;
        next_obj_num += 1;
        Some(n)
    } else {
        None
    };
    offsets.push((catalog_num, 0, output.len()));
    write_catalog_object(&mut output, catalog_num, pages_num, outline_num)?;
    if let (Some(outline), Some(outline_num)) = (&doc.outline, outline_num) {
        offsets.push((outline_num, 0, output.len()));
        write_outline_object(
            &mut output,
            outline_num,
            outline,
            catalog_num,
            &mut offsets,
            &mut next_obj_num,
        )?;
    }
    let xref_offset = output.len();
    let max_obj = next_obj_num - 1;
    write_xref_table(&mut output, &offsets, max_obj)?;
    write_trailer(&mut output, catalog_num, info_num, xref_offset, max_obj + 1)?;
    output.write_all(PDF_EOF.as_bytes())?;
    Ok(output)
}
/// Finds the first occurrence of a byte pattern.
fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    for i in 0..=(haystack.len() - needle.len()) {
        if &haystack[i..i + needle.len()] == needle {
            return Some(i);
        }
    }
    None
}
/// Returns true when the header text describes an image XObject.
fn is_image_object(header: &str) -> bool {
    let has_subtype_image = header.contains("/Subtype /Image") || header.contains("/Subtype/Image");
    has_subtype_image && header.contains("/Width") && header.contains("/Height")
}
/// Reads a numeric field that follows a key in the object header.
fn header_number(header: &str, key: &str) -> Option<f32> {
    let key_pos = header.find(key)?;
    let rest = &header[key_pos + key.len()..];
    let trimmed = rest.trim_start_matches(|c: char| c.is_whitespace());
    let end = trimmed
        .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .unwrap_or(trimmed.len());
    let token = trimmed[..end].trim();
    token.parse::<f32>().ok()
}
/// Collects images referenced from page resources.
fn collect_images(doc: &FPdf) -> Vec<FImage> {
    let mut images = Vec::new();
    for part in &doc.package.parts {
        let data = &part.data;
        let header_region_end = std::cmp::min(data.len(), 1024);
        let header = String::from_utf8_lossy(&data[..header_region_end]).to_string();
        if !is_image_object(&header) {
            continue;
        }
        if let Some(img) = parse_image_object(data) {
            images.push(img);
        }
    }
    images
}
/// Builds an FImage from a raw image object byte slice.
fn parse_image_object(obj_bytes: &[u8]) -> Option<FImage> {
    let header_region_end = std::cmp::min(obj_bytes.len(), 1024);
    let header = String::from_utf8_lossy(&obj_bytes[..header_region_end]).to_string();
    let mut image = FImage::default();
    image.width = header_number(&header, "/Width")? as u32;
    image.height = header_number(&header, "/Height")? as u32;
    image.bits_per_component = header_number(&header, "/BitsPerComponent").unwrap_or(8.0) as u32;
    if let Some(cs) = crate::extract_dict_value(&header, "ColorSpace") {
        image.color_space = if cs.contains("DeviceGray") {
            FColorSpace::DeviceGray
        } else if cs.contains("DeviceCMYK") {
            FColorSpace::DeviceCMYK
        } else {
            FColorSpace::DeviceRGB
        };
    }
    if let Some(filter) = crate::extract_dict_value(&header, "Filter") {
        image.filter = Some(filter.trim_start_matches('/').to_string());
        image.compressed = true;
    }
    let stream_pos = find_bytes(obj_bytes, b"stream")?;
    let mut after = stream_pos + 6;
    if after <= obj_bytes.len() && obj_bytes[after..].starts_with(b"\r\n") {
        after += 2;
    } else if after <= obj_bytes.len()
        && (obj_bytes[after..].starts_with(b"\n") || obj_bytes[after..].starts_with(b"\r"))
    {
        after += 1;
    }
    let declared_len = header_number(&header, "/Length").map(|v| v as usize);
    let end = match declared_len {
        Some(len) if after + len <= obj_bytes.len() => after + len,
        _ => find_bytes(&obj_bytes[after..], b"endstream")
            .map(|v| after + v)
            .unwrap_or(obj_bytes.len()),
    };
    let mut raw = obj_bytes[after..end].to_vec();
    while raw.last() == Some(&b'\n') || raw.last() == Some(&b'\r') {
        raw.pop();
    }
    image.data = raw;
    Some(image)
}
/// Writes a font object with proper object wrapper.
fn write_font_object(output: &mut Vec<u8>, obj_num: u32, font: &FFont) -> FPdfResult<()> {
    writeln!(output, "{} 0 obj", obj_num)?;
    writeln!(output, "<<")?;
    writeln!(output, "  /Type /Font")?;
    let subtype = match font.font_type {
        FFontType::Type1 => "Type1",
        FFontType::TrueType => "TrueType",
        FFontType::Type3 => "Type3",
        FFontType::Type0 => "Type0",
        FFontType::CIDFontType0 => "CIDFontType0",
        FFontType::CIDFontType2 => "CIDFontType2",
        FFontType::MMType1 => "MMType1",
    };
    writeln!(output, "  /Subtype /{}", subtype)?;
    if let Some(base_font) = &font.base_font {
        writeln!(output, "  /BaseFont /{}", base_font)?;
    }
    let encoding = match &font.encoding {
        FFontEncoding::Standard => "StandardEncoding".to_string(),
        FFontEncoding::MacRoman => "MacRomanEncoding".to_string(),
        FFontEncoding::WinAnsi => "WinAnsiEncoding".to_string(),
        FFontEncoding::PDFDoc => "PDFDocEncoding".to_string(),
        FFontEncoding::MacExpert => "MacExpertEncoding".to_string(),
        FFontEncoding::Custom(e) => e.clone(),
        FFontEncoding::PredefinedCMap(e) => e.clone(),
    };
    writeln!(output, "  /Encoding /{}", encoding)?;
    if let Some(first_char) = font.first_char {
        writeln!(output, "  /FirstChar {}", first_char)?;
    }
    if let Some(last_char) = font.last_char {
        writeln!(output, "  /LastChar {}", last_char)?;
    }
    if !font.widths.is_empty() {
        write!(output, "  /Widths [")?;
        for w in &font.widths {
            write!(output, " {}", w)?;
        }
        writeln!(output, " ]")?;
    }
    if let Some(default_width) = font.default_width {
        writeln!(output, "  /MissingWidth {}", default_width)?;
    }
    writeln!(output, ">>")?;
    writeln!(output, "endobj")?;
    Ok(())
}
/// Writes an image XObject.
fn write_image_object(output: &mut Vec<u8>, obj_num: u32, image: &FImage) -> FPdfResult<()> {
    let color_space = match image.color_space {
        FColorSpace::DeviceGray => "DeviceGray",
        FColorSpace::DeviceCMYK => "DeviceCMYK",
        FColorSpace::Indexed => "Indexed",
        FColorSpace::Separation => "Separation",
        FColorSpace::DeviceN => "DeviceN",
        FColorSpace::ICCBased => "ICCBased",
        FColorSpace::Pattern => "Pattern",
        FColorSpace::Lab => "Lab",
        FColorSpace::CalGray => "CalGray",
        FColorSpace::CalRGB => "CalRGB",
        _ => "DeviceRGB",
    };
    writeln!(output, "{} 0 obj", obj_num)?;
    writeln!(output, "<<")?;
    writeln!(output, "  /Type /XObject")?;
    writeln!(output, "  /Subtype /Image")?;
    writeln!(output, "  /Width {}", image.width)?;
    writeln!(output, "  /Height {}", image.height)?;
    if !image.is_mask {
        writeln!(output, "  /BitsPerComponent {}", image.bits_per_component)?;
        writeln!(output, "  /ColorSpace /{}", color_space)?;
    }
    if let Some(filter) = &image.filter {
        writeln!(output, "  /Filter /{}", filter.trim_start_matches('/'))?;
    }
    if image.interpolation == crate::FImageInterpolation::Interpolate {
        writeln!(output, "  /Interpolate true")?;
    }
    if image.is_mask {
        writeln!(output, "  /ImageMask true")?;
    }
    if let Some(decode) = &image.decode {
        write!(output, "  /Decode [")?;
        for v in decode {
            write!(output, " {}", v)?;
        }
        writeln!(output, " ]")?;
    }
    if let Some(sm) = image.soft_mask {
        writeln!(output, "  /SMask {} 0 R", sm)?;
    }
    writeln!(output, "  /Length {}", image.data.len())?;
    writeln!(output, ">>")?;
    writeln!(output, "stream")?;
    output.write_all(&image.data)?;
    writeln!(output)?;
    writeln!(output, "endstream")?;
    writeln!(output, "endobj")?;
    Ok(())
}
/// Writes a page object with proper object wrapper.
fn write_page_object(
    output: &mut Vec<u8>,
    obj_num: u32,
    page: &FPage,
    parent_num: u32,
    font_nums: &[u32],
    image_nums: &[u32],
    page_index: usize,
    offsets: &mut Vec<(u32, u32, usize)>,
    next_obj_num: &mut u32,
) -> FPdfResult<()> {
    writeln!(output, "{} 0 obj", obj_num)?;
    writeln!(output, "<<")?;
    writeln!(output, "  /Type /Page")?;
    writeln!(output, "  /Parent {} 0 R", parent_num)?;
    writeln!(
        output,
        "  /MediaBox [{} {} {} {}]",
        page.media_box[0], page.media_box[1], page.media_box[2], page.media_box[3]
    )?;
    if let Some(crop_box) = page.crop_box {
        writeln!(
            output,
            "  /CropBox [{} {} {} {}]",
            crop_box[0], crop_box[1], crop_box[2], crop_box[3]
        )?;
    }
    if page.rotation != 0 {
        writeln!(output, "  /Rotate {}", page.rotation)?;
    }
    writeln!(output, "  /Resources <<")?;
    writeln!(output, "    /Font <<")?;
    for (i, &font_num) in font_nums.iter().enumerate() {
        writeln!(output, "      /F{} {} 0 R", i + 1, font_num)?;
    }
    writeln!(output, "    >>")?;
    if !image_nums.is_empty() {
        writeln!(output, "    /XObject <<")?;
        for (i, &image_num) in image_nums.iter().enumerate() {
            let name = if i == 0 {
                "Im1".to_string()
            } else {
                format!("Im{}", i + 1)
            };
            let _ = page_index;
            writeln!(output, "      /{} {} 0 R", name, image_num)?;
        }
        writeln!(output, "    >>")?;
    }
    writeln!(output, "  >>")?;
    let content_num = *next_obj_num;
    *next_obj_num += 1;
    writeln!(output, "  /Contents {} 0 R", content_num)?;
    writeln!(output, ">>")?;
    writeln!(output, "endobj")?;
    offsets.push((content_num, 0, output.len()));
    writeln!(output, "{} 0 obj", content_num)?;
    writeln!(output, "<<")?;
    writeln!(
        output,
        "  /Length {}",
        page.contents.iter().map(|c| c.data.len()).sum::<usize>()
    )?;
    writeln!(output, ">>")?;
    writeln!(output, "stream")?;
    for content in &page.contents {
        output.write_all(&content.data)?;
    }
    writeln!(output, "endstream")?;
    writeln!(output, "endobj")?;
    Ok(())
}
/// Writes the page tree object.
fn write_page_tree_object(output: &mut Vec<u8>, obj_num: u32, page_nums: &[u32]) -> FPdfResult<()> {
    writeln!(output, "{} 0 obj", obj_num)?;
    writeln!(output, "<<")?;
    writeln!(output, "  /Type /Pages")?;
    writeln!(output, "  /Kids [")?;
    for &num in page_nums {
        writeln!(output, "    {} 0 R", num)?;
    }
    writeln!(output, "  ]")?;
    writeln!(output, "  /Count {}", page_nums.len())?;
    writeln!(output, ">>")?;
    writeln!(output, "endobj")?;
    Ok(())
}
/// Writes the info object.
fn write_info_object(output: &mut Vec<u8>, obj_num: u32, metadata: &FMetadata) -> FPdfResult<()> {
    writeln!(output, "{} 0 obj", obj_num)?;
    writeln!(output, "<<")?;
    if let Some(title) = &metadata.title {
        writeln!(output, "  /Title ({})", encode_pdf_string(title))?;
    }
    if let Some(author) = &metadata.author {
        writeln!(output, "  /Author ({})", encode_pdf_string(author))?;
    }
    if let Some(subject) = &metadata.subject {
        writeln!(output, "  /Subject ({})", encode_pdf_string(subject))?;
    }
    if let Some(keywords) = &metadata.keywords {
        writeln!(output, "  /Keywords ({})", encode_pdf_string(keywords))?;
    }
    if let Some(creator) = &metadata.creator {
        writeln!(output, "  /Creator ({})", encode_pdf_string(creator))?;
    }
    if let Some(producer) = &metadata.producer {
        writeln!(output, "  /Producer ({})", encode_pdf_string(producer))?;
    }
    if let Some(creation_date) = &metadata.creation_date {
        writeln!(output, "  /CreationDate ({})", creation_date)?;
    }
    if let Some(mod_date) = &metadata.modification_date {
        writeln!(output, "  /ModDate ({})", mod_date)?;
    }
    writeln!(output, ">>")?;
    writeln!(output, "endobj")?;
    Ok(())
}
/// Writes the catalog object.
fn write_catalog_object(
    output: &mut Vec<u8>,
    obj_num: u32,
    pages_num: u32,
    outline_num: Option<u32>,
) -> FPdfResult<()> {
    writeln!(output, "{} 0 obj", obj_num)?;
    writeln!(output, "<<")?;
    writeln!(output, "  /Type /Catalog")?;
    writeln!(output, "  /Pages {} 0 R", pages_num)?;
    if let Some(outline_num) = outline_num {
        writeln!(output, "  /Outlines {} 0 R", outline_num)?;
    }
    writeln!(output, "  /PageMode /UseNone")?;
    writeln!(output, ">>")?;
    writeln!(output, "endobj")?;
    Ok(())
}
/// Writes the outline object.
fn write_outline_object(
    output: &mut Vec<u8>,
    obj_num: u32,
    outline: &FOutline,
    _parent: u32,
    offsets: &mut Vec<(u32, u32, usize)>,
    next_obj_num: &mut u32,
) -> FPdfResult<()> {
    writeln!(output, "{} 0 obj", obj_num)?;
    writeln!(output, "<<")?;
    writeln!(output, "  /Type /Outlines")?;
    if !outline.items.is_empty() {
        writeln!(output, "  /First {} 0 R", obj_num + 1)?;
        writeln!(
            output,
            "  /Last {} 0 R",
            obj_num + outline.items.len() as u32
        )?;
    }
    writeln!(output, "  /Count {}", outline.items.len())?;
    writeln!(output, ">>")?;
    writeln!(output, "endobj")?;
    for (i, item) in outline.items.iter().enumerate() {
        let item_num = *next_obj_num;
        *next_obj_num += 1;
        offsets.push((item_num, 0, output.len()));
        writeln!(output, "{} 0 obj", item_num)?;
        writeln!(output, "<<")?;
        writeln!(output, "  /Title ({})", encode_pdf_string(&item.title))?;
        if let Some(dest) = &item.dest {
            writeln!(output, "  /Dest {}", dest)?;
        }
        writeln!(output, "  /Parent {} 0 R", obj_num)?;
        if i > 0 {
            writeln!(output, "  /Prev {} 0 R", item_num - 1)?;
        }
        if i < outline.items.len() - 1 {
            writeln!(output, "  /Next {} 0 R", item_num + 1)?;
        }
        writeln!(output, ">>")?;
        writeln!(output, "endobj")?;
    }
    Ok(())
}
/// Writes the xref table.
fn write_xref_table(
    output: &mut Vec<u8>,
    offsets: &[(u32, u32, usize)],
    max_obj: u32,
) -> FPdfResult<()> {
    writeln!(output, "xref")?;
    writeln!(output, "0 {}", max_obj + 1)?;
    writeln!(output, "{:010} {:05} f ", 0, 0)?;
    let mut sorted = offsets.to_vec();
    sorted.sort_by_key(|&(num, _, _)| num);
    let mut current = 1;
    for (num, r#gen, offset) in sorted {
        while current < num {
            writeln!(output, "{:010} {:05} f ", 0, 0)?;
            current += 1;
        }
        writeln!(output, "{:010} {:05} n ", offset, r#gen)?;
        current = num + 1;
    }
    Ok(())
}
/// Writes the trailer.
fn write_trailer(
    output: &mut Vec<u8>,
    catalog_num: u32,
    info_num: u32,
    xref_offset: usize,
    size: u32,
) -> FPdfResult<()> {
    writeln!(output, "trailer")?;
    writeln!(output, "<<")?;
    writeln!(output, "  /Size {}", size)?;
    writeln!(output, "  /Root {} 0 R", catalog_num)?;
    writeln!(output, "  /Info {} 0 R", info_num)?;
    writeln!(output, ">>")?;
    writeln!(output, "startxref")?;
    writeln!(output, "{}", xref_offset)?;
    Ok(())
}
