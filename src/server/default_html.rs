use std::collections::HashMap;

use crate::config::RouteConfig;
use crate::core::Response;
use crate::server::handler::utils::default_reason_phrase;

/// Render a simple default index listing configured routes.
pub fn default_index_response(routes: &HashMap<String, RouteConfig>) -> Response {
    let mut body = String::from(
        "<!DOCTYPE html>\
         <html lang=\"en\">\
         <head>\
           <meta charset=\"utf-8\">\
           <title>Server Index</title>\
           <style>\
             body { font-family: sans-serif; margin: 2rem; }\
             h1 { margin-bottom: 1rem; }\
             ul { list-style: none; padding: 0; }\
             li { margin: 0.5rem 0; }\
             code { background: #f0f0f0; padding: 0.2rem 0.4rem; border-radius: 4px; }\
           </style>\
         </head>\
         <body>\
           <h1>Configured Routes</h1>\
           <p>The requested resource is not configured. Available routes:</p>\
           <ul>",
    );

    if routes.is_empty() {
        body.push_str("<li><em>No routes configured</em></li>");
    } else {
        for (route, cfg) in routes {
            let description = if cfg.redirect.is_some() {
                "redirect"
            } else if cfg.directory.is_some() {
                "directory"
            } else if cfg.filename.is_some() {
                "file"
            } else if cfg.upload_dir.is_some() {
                "upload"
            } else {
                "unknown"
            };
            body.push_str(&format!(
                "<li><code>{}</code> &mdash; {}</li>",
                route, description
            ));
        }
    }

    body.push_str(
        "  </ul>\
         </body>\
         </html>",
    );

    Response::new(200, default_reason_phrase(200))
        .header("Content-Type", "text/html; charset=utf-8")
        .with_body(body)
}
