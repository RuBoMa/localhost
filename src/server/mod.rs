mod server;
mod default_html;
mod handler;

pub use server::Server;
pub use default_html::{DEFAULT_404_PAGE, DEFAULT_WELCOME_PAGE};
pub use handler::serve_static_file;