/// Utility functions for path handling and other helpers
/// Strip Windows UNC (\\?\) prefix from a path string
pub fn strip_unc_prefix(path: &str) -> &str {
    path.strip_prefix(r"\\?\").unwrap_or(path)
}

/// Strip Windows UNC (\\?\) prefix from a PathBuf, returning a new PathBuf
pub fn strip_unc_pathbuf(path: &std::path::Path) -> std::path::PathBuf {
    if let Some(s) = path.to_str() {
        std::path::PathBuf::from(strip_unc_prefix(s))
    } else {
        path.to_path_buf()
    }
}
