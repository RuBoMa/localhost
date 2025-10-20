mod server;
mod default_html;
mod handler;

pub use server::Server;
pub use default_html::{default_404_response, default_welcome_response, default_method_not_allowed_response};
pub use handler::serve_static_file;