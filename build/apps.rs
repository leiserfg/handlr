// This file exists solely to trick build script into working
// These types are used by cli.rs, which cannot be transitively imported
// because they rely on their own dependencies and so on

use std::error::Error;
use std::ffi::OsString;

pub struct SystemApps;
pub struct DesktopEntry {
    pub name: String,
}

impl SystemApps {
    pub fn get_entries(
    ) -> Result<impl Iterator<Item = (OsString, DesktopEntry)>, Box<dyn Error>>
    {
        Ok(vec![(
            OsString::new(),
            DesktopEntry {
                name: String::new(),
            },
        )]
        .into_iter())
    }
}
