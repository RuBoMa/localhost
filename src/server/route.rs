use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,                        // e.g. "/test"
    pub methods: HashSet<String>,           // e.g. GET, POST
    pub root: Option<PathBuf>,              // Filesystem root for static content
    pub default_file: Option<String>,       // e.g. index.html
    pub cgi_extension: Option<String>,      // e.g. ".php"
    pub redirection: Option<String>,        // e.g. "/new-path"
    pub directory_listing: bool,            // true = allow dir listing
}
