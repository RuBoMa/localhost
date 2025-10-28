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
           <link rel=\"stylesheet\" type=\"text/css\" href=\"/static/style.css\">\
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
            let methods = if let Some(methods) = &cfg.methods {
                methods.join(", ")
            } else {
                "ALL".to_string()
            };
        
            body.push_str(&format!(
                "<li><code>[{}] - <a href={}>{}</a></code></li>",
                methods, route, route
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
