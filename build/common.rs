// This file exists solely to trick build script into working
// These types are used by cli.rs, which cannot be transitively imported
// because they rely on their own dependencies and so on

pub type DesktopHandler = String;
pub type MimeOrExtension = String;
pub type UserPath = String;

pub fn mime_types() -> Vec<String> {
    vec!["".to_string()]
}
