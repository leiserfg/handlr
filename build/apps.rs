// This file exists solely to trick build script into working
// These types are used by cli.rs, which cannot be transitively imported
// because they rely on their own dependencies and so on

use std::error::Error;

pub struct SystemApps;
pub struct DesktopEntry {
    pub name: String,
}

impl SystemApps {
    pub fn get_entries(
    ) -> Result<impl Iterator<Item = (String, DesktopEntry)>, Box<dyn Error>>
    {
        let name = "".to_string();
        Ok(vec![(name.clone(), DesktopEntry { name })].into_iter())
    }
}
