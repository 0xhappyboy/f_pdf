#[cfg(test)]
mod tests {
    use f_pdf::{
        FAnnotation, FAnnotationFlags, FBorderStyle, FContent, FFont, FFontEncoding, FFontType,
        FImage, FImageInterpolation, FMetadata, FOutline, FOutlineItem, FPage, FPdf,
    };
    use std::io::Read;
    const TEST_DOWNLOAD_IMAGE_URL: &str = "https://picsum.photos/id/237/200/200";
    /// Output path for demo PDF file.
    fn demo_path() -> std::path::PathBuf {
        std::path::PathBuf::from(r"C:\Users\admin\Downloads\demo.pdf")
    }
    /// Builds a simple font.
    fn make_font(name: &str, base_font: &str, font_type: FFontType) -> FFont {
        FFont {
            name: name.to_string(),
            font_type,
            base_font: Some(base_font.to_string()),
            encoding: FFontEncoding::WinAnsi,
            descriptor: None,
            to_unicode: None,
            first_char: Some(32),
            last_char: Some(255),
            widths: vec![500.0; 224],
            default_width: Some(500.0),
            cid_system_info: None,
            descendant_fonts: Vec::new(),
            raw_dict: None,
        }
    }
    /// Builds a page with the given content.
    fn make_page(content: &str, page_number: u32, width: f32, height: f32) -> FPage {
        FPage {
            page_number,
            media_box: [0.0, 0.0, width, height],
            crop_box: Some([50.0, 50.0, width - 50.0, height - 50.0]),
            rotation: 0,
            contents: vec![FContent {
                data: content.as_bytes().to_vec(),
                compressed: false,
                filters: Vec::new(),
            }],
            resources: Some("<< /Font << /F1 1 0 R /F2 2 0 R >> >>".to_string()),
            annotations: Vec::new(),
            raw_dict: None,
        }
    }
    /// Downloads an image from URL and returns raw bytes.
    fn download_image(url: &str) -> Vec<u8> {
        ureq::get(url)
            .call()
            .expect("failed to download image")
            .into_body()
            .into_reader()
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
            .expect("failed to read image data")
    }
    /// Decodes arbitrary image bytes into raw RGB pixels.
    fn decode_to_rgb(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
        let img = image::load_from_memory(bytes).expect("failed to decode image");
        let rgb = img.to_rgb8();
        let (w, h) = (rgb.width(), rgb.height());
        (w, h, rgb.into_raw())
    }
    /// Builds an FImage with raw RGB pixel data.
    fn make_image_from_rgb(width: u32, height: u32, data: Vec<u8>) -> FImage {
        FImage {
            width,
            height,
            bits_per_component: 8,
            color_space: f_pdf::FColorSpace::DeviceRGB,
            data,
            compressed: false,
            filter: None,
            interpolation: FImageInterpolation::None,
            is_mask: false,
            soft_mask: None,
            decode: None,
            raw_dict: None,
        }
    }
    /// Encodes an FImage into a raw image XObject byte buffer.
    fn image_object_bytes(obj_num: u32, image: &FImage) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(format!("{} 0 obj\n", obj_num).as_bytes());
        out.extend_from_slice(b"<<\n");
        out.extend_from_slice(b"  /Type /XObject\n");
        out.extend_from_slice(b"  /Subtype /Image\n");
        out.extend_from_slice(format!("  /Width {}\n", image.width).as_bytes());
        out.extend_from_slice(format!("  /Height {}\n", image.height).as_bytes());
        out.extend_from_slice(
            format!("  /BitsPerComponent {}\n", image.bits_per_component).as_bytes(),
        );
        out.extend_from_slice(b"  /ColorSpace /DeviceRGB\n");
        if let Some(filter) = &image.filter {
            out.extend_from_slice(format!("  /Filter /{}\n", filter).as_bytes());
        }
        out.extend_from_slice(format!("  /Length {}\n", image.data.len()).as_bytes());
        out.extend_from_slice(b">>\n");
        out.extend_from_slice(b"stream\n");
        out.extend_from_slice(&image.data);
        out.extend_from_slice(b"\nendstream\n");
        out.extend_from_slice(b"endobj");
        out
    }
    /// Writes a complex PDF document covering all features.
    #[test]
    fn write_demo_pdf() {
        let mut doc = FPdf::new_empty();
        doc.metadata = FMetadata {
            title: Some("Demo PDF Document".to_string()),
            author: Some("f_pdf Engine".to_string()),
            subject: Some("Comprehensive PDF feature demonstration".to_string()),
            keywords: Some("pdf, demo, rust, features".to_string()),
            creator: Some("f_pdf test suite".to_string()),
            producer: Some("f_pdf 0.1.0".to_string()),
            creation_date: Some("D:20240101120000".to_string()),
            modification_date: Some("D:20240101120000".to_string()),
            trapped: Some("False".to_string()),
            custom: std::collections::HashMap::new(),
        };
        doc.fonts
            .push(make_font("F1", "Helvetica", FFontType::Type1));
        doc.fonts
            .push(make_font("F2", "Courier", FFontType::TrueType));
        let title_content = r#"
            0.5 0.5 0.5 RG
            2.0 w
            1.0 1.0 1.0 rg
            100 600 400 100 re
            S
            BT
            /F1 24 Tf
            150 650 Td
            (Demo PDF Document) Tj
            ET
            BT
            /F2 12 Tf
            150 620 Td
            (Generated by f_pdf engine) Tj
            ET
        "#;
        let mut page1 = make_page(title_content, 1, 612.0, 792.0);
        page1.annotations.push(FAnnotation {
            annot_type: "Text".to_string(),
            rect: [50.0, 50.0, 70.0, 70.0],
            contents: Some("Title page annotation".to_string()),
            author: Some("Demo".to_string()),
            subject: Some("Note".to_string()),
            creation_date: Some("D:20240101120000".to_string()),
            modification_date: None,
            name: Some("TitleNote".to_string()),
            flags: FAnnotationFlags {
                print: true,
                ..Default::default()
            },
            border: Some(FBorderStyle {
                width: 1.0,
                style: "Solid".to_string(),
                dash_pattern: Vec::new(),
            }),
            color: Some([1.0, 0.0, 0.0]),
            destination: None,
            action: None,
            appearance: None,
            raw_dict: None,
        });
        doc.pages.push(page1);
        let image_bytes = download_image(TEST_DOWNLOAD_IMAGE_URL);
        println!(
            "[image] downloaded {} bytes, first bytes: {:?}",
            image_bytes.len(),
            &image_bytes[..std::cmp::min(16, image_bytes.len())]
        );
        let (img_w, img_h, rgb_data) = decode_to_rgb(&image_bytes);
        println!(
            "[image] decoded to RGB: {}x{}, {} bytes",
            img_w,
            img_h,
            rgb_data.len()
        );
        let image = make_image_from_rgb(img_w, img_h, rgb_data);
        doc.package.set_part(3, 0, image_object_bytes(3, &image));
        let content2 = r#"
            BT
            /F1 16 Tf
            72 720 Td
            (Chapter 1: Introduction) Tj
            ET
            BT
            /F2 11 Tf
            72 700 Td
            14 TL
            (This is the first paragraph of the document.) Tj
            T*
            (It contains multiple lines of text.) Tj
            T*
            (Using Courier font for a typewriter effect.) Tj
            ET
            0.0 0.0 1.0 RG
            1.0 w
            72 680 m
            540 680 l
            S
            q
            200 0 0 200 200 400 cm
            /Im1 Do
            Q
        "#;
        let mut page2 = make_page(content2, 2, 612.0, 792.0);
        page2.resources =
            Some("<< /Font << /F1 1 0 R /F2 2 0 R >> /XObject << /Im1 3 0 R >> >>".to_string());
        doc.pages.push(page2);
        let content3 = r#"
            BT
            /F1 14 Tf
            72 720 Td
            (Chapter 2: Data Table) Tj
            ET
            0.0 0.0 0.0 RG
            0.5 w
            72 600 m
            540 600 l
            540 680 l
            72 680 l
            h
            S
            72 640 m
            540 640 l
            S
            306 600 m
            306 680 l
            S
            BT
            /F1 10 Tf
            80 660 Td
            (Header 1) Tj
            ET
            BT
            /F1 10 Tf
            320 660 Td
            (Header 2) Tj
            ET
            BT
            /F1 10 Tf
            80 620 Td
            (Data 1) Tj
            ET
            BT
            /F1 10 Tf
            320 620 Td
            (Data 2) Tj
            ET
        "#;
        let mut page3 = make_page(content3, 3, 612.0, 792.0);
        page3.annotations.push(FAnnotation {
            annot_type: "Link".to_string(),
            rect: [72.0, 600.0, 306.0, 680.0],
            contents: None,
            author: None,
            subject: None,
            creation_date: None,
            modification_date: None,
            name: None,
            flags: FAnnotationFlags::default(),
            border: Some(FBorderStyle {
                width: 0.0,
                style: "Solid".to_string(),
                dash_pattern: Vec::new(),
            }),
            color: None,
            destination: Some("[1 0 R /XYZ 0 720 0]".to_string()),
            action: Some("<< /S /GoTo /D [1 0 R /XYZ 0 720 0] >>".to_string()),
            appearance: None,
            raw_dict: None,
        });
        doc.pages.push(page3);
        let mut outline = FOutline::default();
        outline.items = vec![
            FOutlineItem {
                title: "Title Page".to_string(),
                dest: Some("[1 0 R /XYZ 0 720 0]".to_string()),
                action: None,
                children: Vec::new(),
                parent: None,
                prev: None,
                next: Some(2),
                first: None,
                last: None,
                count: None,
                color: None,
                font_flags: Some(2),
            },
            FOutlineItem {
                title: "Chapter 1: Introduction".to_string(),
                dest: Some("[2 0 R /XYZ 0 720 0]".to_string()),
                action: None,
                children: vec![FOutlineItem {
                    title: "First Paragraph".to_string(),
                    dest: Some("[2 0 R /XYZ 0 700 0]".to_string()),
                    ..Default::default()
                }],
                parent: None,
                prev: Some(1),
                next: Some(3),
                first: Some(4),
                last: Some(4),
                count: Some(1),
                color: Some([0.0, 0.0, 1.0]),
                font_flags: None,
            },
            FOutlineItem {
                title: "Chapter 2: Data Table".to_string(),
                dest: Some("[3 0 R /XYZ 0 720 0]".to_string()),
                action: None,
                children: Vec::new(),
                parent: None,
                prev: Some(2),
                next: None,
                first: None,
                last: None,
                count: None,
                color: None,
                font_flags: None,
            },
        ];
        doc.outline = Some(outline);
        let bytes = doc.to_bytes().expect("failed to serialize document");
        println!("[writer] serialized document: {} bytes", bytes.len());
        std::fs::write(demo_path(), &bytes).expect("failed to write demo.pdf");
        println!("demo.pdf written to {}", demo_path().display());
    }
    /// Reads the demo PDF and prints its contents.
    #[test]
    fn read_demo_pdf() {
        let doc = FPdf::with_path(demo_path()).expect("failed to read demo.pdf");
        println!("=== PDF Metadata ===");
        println!("Title: {:?}", doc.metadata.title);
        println!("Author: {:?}", doc.metadata.author);
        println!("Subject: {:?}", doc.metadata.subject);
        println!("Keywords: {:?}", doc.metadata.keywords);
        println!("Creator: {:?}", doc.metadata.creator);
        println!("Producer: {:?}", doc.metadata.producer);
        println!("Creation Date: {:?}", doc.metadata.creation_date);
        println!("Modification Date: {:?}", doc.metadata.modification_date);
        println!();
        println!("=== Document Info ===");
        println!("Version: {}", doc.version);
        println!("Page Count: {}", doc.page_count());
        println!("Font Count: {}", doc.fonts.len());
        println!();
        println!("=== Fonts ===");
        for (i, font) in doc.fonts.iter().enumerate() {
            println!(
                "Font {}: name={}, base_font={:?}, type={:?}",
                i + 1,
                font.name,
                font.base_font,
                font.font_type
            );
        }
        println!();
        println!("=== Pages ===");
        for page in &doc.pages {
            println!("Page {}:", page.page_number);
            println!("  MediaBox: {:?}", page.media_box);
            println!("  CropBox: {:?}", page.crop_box);
            println!("  Rotation: {}", page.rotation);
            println!("  Contents: {} stream(s)", page.contents.len());
            println!("  Annotations: {}", page.annotations.len());
            for (i, annot) in page.annotations.iter().enumerate() {
                println!(
                    "    Annotation {}: type={}, rect={:?}",
                    i + 1,
                    annot.annot_type,
                    annot.rect
                );
            }
        }
        println!();
        println!("=== Outline ===");
        if let Some(outline) = &doc.outline {
            for (i, item) in outline.items.iter().enumerate() {
                println!("  {}: {}", i + 1, item.title);
                for (j, child) in item.children.iter().enumerate() {
                    println!("    {}.{}: {}", i + 1, j + 1, child.title);
                }
            }
        } else {
            println!("  No outline");
        }
        println!();
        println!("=== Plain Text ===");
        println!("{}", doc.plain_text());
    }
}
