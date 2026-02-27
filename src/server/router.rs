use crate::config::RouteConfig;
use std::collections::HashMap;

/// Nginx-style route matcher:
/// - Finds the **longest prefix** that matches the URI.
/// - Ensures prefix boundaries are respected ("/static" doesn’t match "/statics").
/// - Returns both the matched prefix and its RouteConfig.
pub fn match_route<'a>(
    routes: &'a HashMap<String, RouteConfig>,
    uri: &str,
) -> Option<(&'a str, &'a RouteConfig)> {
    let mut best_match: Option<(&str, &RouteConfig)> = None;
    let mut best_len = 0;

    for (prefix, cfg) in routes.iter() {
        // Must start with prefix, and match a boundary or be exact
        if uri == prefix || uri.starts_with(prefix) {
            // Ensure prefix boundary ("/static" shouldn't match "/static2")
            if uri == prefix
                || uri
                    .strip_prefix(prefix)
                    .map_or(false, |r| r.starts_with('/') || r.is_empty())
            {
                if prefix.len() > best_len {
                    best_len = prefix.len();
                    best_match = Some((prefix.as_str(), cfg));
                }
            }
        }
    }

    best_match
}

#[cfg(test)]
mod tests {
    use super::match_route;
    use crate::config::RouteConfig;
    use std::collections::HashMap;

    fn route_config() -> RouteConfig {
        RouteConfig {
            filename: None,
            directory: None,
            directory_listing: false,
            methods: None,
            redirect: None,
            upload_dir: None,
        }
    }

    #[test]
    fn picks_longest_matching_prefix() {
        let mut routes = HashMap::new();
        routes.insert("/".to_string(), route_config());
        routes.insert("/static".to_string(), route_config());
        routes.insert("/static/images".to_string(), route_config());

        let matched = match_route(&routes, "/static/images/logo.png").map(|(prefix, _)| prefix);
        assert_eq!(matched, Some("/static/images"));
    }

    #[test]
    fn respects_route_boundaries() {
        let mut routes = HashMap::new();
        routes.insert("/static".to_string(), route_config());

        let matched = match_route(&routes, "/static2").map(|(prefix, _)| prefix);
        assert_eq!(matched, None);
    }

    #[test]
    fn matches_exact_prefix() {
        let mut routes = HashMap::new();
        routes.insert("/hello".to_string(), route_config());

        let matched = match_route(&routes, "/hello").map(|(prefix, _)| prefix);
        assert_eq!(matched, Some("/hello"));
    }
}
