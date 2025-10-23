use std::{fs, path::Path};
use crate::core::Response;
pub fn generate_directory_listing(dir: &Path, base_url_path: &str) -> Response {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html>
<head><title>Directory Listing</title></head>
<body>
<h1>Directory Listing</h1>
<ul>"#,
    );

    // Add parent directory link if not at root
    if let Some(_) = dir.parent() {
        // Parent link should go one level up from current base_url_path
        // For example, if base_url_path = "/files/subdir", parent is "/files"
        let parent_path = if let Some(pos) = base_url_path.rfind('/') {
            &base_url_path[..pos]
        } else {
            "/"
        };
        html.push_str(&format!(
            "<li><a href=\"{}\">{}</a></li>",
            parent_path, ".."
        ));
    }

    if let Ok(entries) = fs::read_dir(dir) {
        let mut names: Vec<String> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();

        names.sort();

        for name in names {
            let encoded_name = url_encode(&name);
            // Prepend base_url_path, making sure there's exactly one slash between
            let url = if base_url_path.ends_with('/') {
                format!("{}{}", base_url_path, encoded_name)
            } else {
                format!("{}/{}", base_url_path, encoded_name)
            };
            html.push_str(&format!("<li><a href=\"{}\">{}</a></li>", url, name));
        }
    } else {
        html.push_str("<li><em>Could not read directory</em></li>");
    }

    html.push_str("</ul></body></html>");

    Response::new(200, "OK")
        .header("Content-Type", "text/html")
        .with_body(html)
}

fn url_encode(input: &str) -> String {
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
