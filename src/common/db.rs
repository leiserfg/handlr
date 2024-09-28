use itertools::Itertools;

use crate::error::Result;
use std::io::Write;

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

pub fn autocomplete<W: Write>(writer: &mut W) -> Result<()> {
    mime_db::EXTENSIONS
        .iter()
        .try_for_each(|(ext, _)| writeln!(writer, ".{}", ext))?;

    mime_types()
        .iter()
        .try_for_each(|mime| writeln!(writer, "{}", mime))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autocomplete_mimes_and_extensions() -> Result<()> {
        let mut buffer = Vec::new();
        autocomplete(&mut buffer)?;
        goldie::assert!(String::from_utf8(buffer)?);
        Ok(())
    }
}
