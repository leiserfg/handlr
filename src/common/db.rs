use itertools::Itertools;

static CUSTOM_MIMES: &[&str] = &[
    "inode/directory",
    "x-scheme-handler/http",
    "x-scheme-handler/https",
    "x-scheme-handler/terminal",
];

/// Helper function to get a list of known mime types
pub fn mime_types() -> Vec<String> {
    CUSTOM_MIMES
        .iter()
        .map(|s| s.to_string())
        .chain(
            mime_db::TYPES
                .into_iter()
                .map(|(mime, _, _)| mime.to_string()),
        )
        .collect_vec()
}
