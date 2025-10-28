pub fn url_decode(input: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '%' => {
                // Expect two hex digits after '%'
                let hi = chars.next()?.to_digit(16)?;
                let lo = chars.next()?.to_digit(16)?;
                bytes.push((hi << 4 | lo) as u8);
            }
            '+' => {
                // Optional: treat '+' as space
                bytes.push(b' ');
            }
            _ => {
                // Safe ASCII character
                bytes.push(c as u8);
            }
        }
    }

    // Convert bytes back to UTF-8 string
    String::from_utf8(bytes).ok()
}

pub fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                // Safe characters, push as is
                encoded.push(b as char);
            }
            _ => {
                // Percent-encode everything else
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}
