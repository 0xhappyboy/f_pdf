//! Minimal text and parsing helpers.
pub(crate) fn extract_delimited_text(
    s: &str,
    start_delim: &str,
    end_delim: &str,
) -> Option<String> {
    let start = s.find(start_delim)?;
    let after = start + start_delim.len();
    let end_rel = s[after..].find(end_delim)?;
    Some(s[after..after + end_rel].to_string())
}
/// Returns the value of a dictionary entry by key.
pub(crate) fn extract_dict_value(dict: &str, key: &str) -> Option<String> {
    let needle = format!("/{}", key);
    let start = dict.find(&needle)?;
    let after = start + needle.len();
    // Skip whitespace
    let mut pos = after;
    while pos < dict.len() && dict[pos..].starts_with(char::is_whitespace) {
        pos += 1;
    }
    // Check if this is a reference (num gen R pattern)
    let rest = &dict[pos..];
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() >= 3 && parts[2] == "R" {
        // Reference - return full "num gen R"
        return Some(format!("{} {} R", parts[0], parts[1]));
    }
    // Find the end of the value
    let end = find_value_end(dict, pos);
    Some(dict[pos..end].trim().to_string())
}
/// Finds the end position of a PDF value starting at pos.
fn find_value_end(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return pos;
    }
    let bytes = s.as_bytes();
    match bytes[pos] {
        b'(' => {
            // String literal - find matching close paren
            let mut depth = 1;
            let mut i = pos + 1;
            while i < s.len() && depth > 0 {
                match bytes[i] {
                    b'(' => depth += 1,
                    b')' => depth -= 1,
                    b'\\' => i += 1, // Skip escaped char
                    _ => {}
                }
                i += 1;
            }
            i
        }
        b'[' => {
            // Array - find matching close bracket
            let mut depth = 1;
            let mut i = pos + 1;
            while i < s.len() && depth > 0 {
                match bytes[i] {
                    b'[' => depth += 1,
                    b']' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            i
        }
        b'<' => {
            // Hex string or dictionary
            if pos + 1 < s.len() && bytes[pos + 1] == b'<' {
                // Dictionary
                let mut depth = 1;
                let mut i = pos + 2;
                while i < s.len() && depth > 0 {
                    if i + 1 < s.len() {
                        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
                            depth += 1;
                            i += 1;
                        } else if bytes[i] == b'>' && bytes[i + 1] == b'>' {
                            depth -= 1;
                            i += 1;
                        }
                    }
                    i += 1;
                }
                i
            } else {
                // Hex string
                match s[pos..].find('>') {
                    Some(v) => pos + v + 1,
                    None => s.len(),
                }
            }
        }
        b'/' => {
            // Name object - find end of name
            let mut i = pos + 1;
            while i < s.len()
                && !bytes[i].is_ascii_whitespace()
                && bytes[i] != b'/'
                && bytes[i] != b'['
                && bytes[i] != b']'
                && bytes[i] != b'('
                && bytes[i] != b'<'
                && bytes[i] != b'>'
            {
                i += 1;
            }
            i
        }
        _ => {
            // Number, keyword, or other - find whitespace or delimiter
            let mut i = pos;
            while i < s.len()
                && !bytes[i].is_ascii_whitespace()
                && bytes[i] != b'/'
                && bytes[i] != b'['
                && bytes[i] != b']'
                && bytes[i] != b'('
                && bytes[i] != b'<'
                && bytes[i] != b'>'
            {
                i += 1;
            }
            i
        }
    }
}
/// Collects all objects with a specific pattern.
pub(crate) fn collect_objects(content: &str, pattern: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut cursor = 0;
    while let Some(rel_start) = content[cursor..].find(pattern) {
        let start = cursor + rel_start;
        let end = find_value_end(content, start + pattern.len());
        result.push(content[start..end].to_string());
        cursor = end;
    }
    result
}
/// Decodes a PDF string literal, handling escape sequences.
pub(crate) fn decode_pdf_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000C}'),
                Some('(') => result.push('('),
                Some(')') => result.push(')'),
                Some('\\') => result.push('\\'),
                Some(c) if c.is_ascii_digit() => {
                    // Octal escape
                    let mut octal = String::from(c);
                    for _ in 0..2 {
                        if let Some(&next) = chars.peek() {
                            if next.is_ascii_digit() && next < '8' {
                                octal.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }
                    }
                    if let Ok(val) = u32::from_str_radix(&octal, 8) {
                        if let Some(ch) = char::from_u32(val) {
                            result.push(ch);
                        }
                    }
                }
                Some(c) => result.push(c),
                None => {}
            }
        } else {
            result.push(c);
        }
    }
    result
}
/// Encodes text for inclusion in a PDF string literal.
pub(crate) fn encode_pdf_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '(' => result.push_str("\\("),
            ')' => result.push_str("\\)"),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c => result.push(c),
        }
    }
    result
}
/// Decodes a hex string to bytes.
pub(crate) fn decode_hex_string(s: &str) -> Vec<u8> {
    let cleaned: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    let mut result = Vec::new();
    let chars: Vec<char> = cleaned.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let hex = if i + 1 < chars.len() {
            format!("{}{}", chars[i], chars[i + 1])
        } else {
            format!("{}0", chars[i])
        };
        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
            result.push(byte);
        }
        i += 2;
    }
    result
}
/// Encodes bytes as a hex string.
pub(crate) fn encode_hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02X}", b)).collect()
}
/// Checks if a pattern exists in the content.
pub(crate) fn pattern_exists(content: &str, pattern: &str) -> bool {
    content.contains(pattern)
}
/// Extracts a reference in the form "num gen R".
pub(crate) fn extract_reference(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() >= 3 && parts[2] == "R" {
        let num = parts[0].parse().ok()?;
        let r#gen = parts[1].parse().ok()?;
        Some((num, r#gen))
    } else {
        None
    }
}
