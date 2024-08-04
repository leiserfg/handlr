use crate::{
    common::{DesktopHandler, Handleable},
    error::{Error, ErrorKind, Result},
};
use derive_more::{Deref, DerefMut};
use itertools::Itertools;
use mime::Mime;
use serde::{Deserialize, Serialize};
use serde_with::{
    serde_as, DeserializeFromStr, DisplayFromStr, SerializeDisplay,
};
use std::{
    collections::{BTreeMap, VecDeque},
    fmt::Display,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

/// Represents user-configured mimeapps.list file
#[serde_as]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
// IMPORTANT: This ensures missing fields are replaced by a default value rather than making deserialization fail entirely
#[serde(default)]
pub struct MimeApps {
    #[serde(rename = "Added Associations")]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub added_associations: BTreeMap<Mime, DesktopList>,
    #[serde(rename = "Default Applications")]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub default_apps: BTreeMap<Mime, DesktopList>,
}

/// Helper struct for a list of `DesktopHandler`s
#[serde_as]
#[derive(
    Debug,
    Default,
    Clone,
    Deref,
    DerefMut,
    SerializeDisplay,
    DeserializeFromStr,
    PartialEq,
)]
pub struct DesktopList(VecDeque<DesktopHandler>);

impl FromStr for DesktopList {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split(';')
                .filter(|s| !s.is_empty()) // Account for ending/duplicated semicolons
                .unique() // Remove duplicate entries
                .map(DesktopHandler::from_str)
                .collect::<Result<_>>()?,
        ))
    }
}

impl Display for DesktopList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};", self.iter().join(";"))
    }
}

impl MimeApps {
    /// Add a handler to an existing default application association
    pub fn add_handler(&mut self, mime: &Mime, handler: &DesktopHandler) {
        self.default_apps
            .entry(mime.clone())
            .or_default()
            .push_back(handler.clone());
    }

    /// Set a default application association, overwriting any existing association for the same mimetype
    pub fn set_handler(&mut self, mime: &Mime, handler: &DesktopHandler) {
        self.default_apps
            .insert(mime.clone(), DesktopList(vec![handler.clone()].into()));
    }

    /// Entirely remove a given mime's default application association
    pub fn unset_handler(&mut self, mime: &Mime) -> Option<DesktopList> {
        self.default_apps.remove(mime)
    }

    /// Remove a given handler from a given mime's default file associaion
    pub fn remove_handler(
        &mut self,
        mime: &Mime,
        handler: &DesktopHandler,
    ) -> Option<DesktopHandler> {
        let handler_list = self.default_apps.entry(mime.clone()).or_default();

        handler_list
            .iter()
            .position(|x| *x == *handler)
            .and_then(|pos| handler_list.remove(pos))
    }

    /// Get a list of handlers associated with a wildcard mime
    fn get_from_wildcard(&self, mime: &Mime) -> Option<&DesktopList> {
        // Get the handlers that wildcard match the given mime
        let associations = self.default_apps.iter().filter(|(m, _)| {
            wildmatch::WildMatch::new(m.as_ref()).matches(mime.as_ref())
        });

        // Get the length of the longest wildcard that matches
        // Assuming the longest match is the best match
        // Inspired by how globs are handled in xdg spec
        let biggest_wildcard_len = associations
            .clone()
            .map(|(ref m, _)| m.as_ref().len())
            .max()?;

        // Keep only the lists of handlers from associations with the longest wildcards
        // And get the first one, assuming it takes precedence
        // Loosely inspired by how globs are handled in xdg spec
        associations
            .filter(|(ref m, _)| m.as_ref().len() == biggest_wildcard_len)
            .map(|(_, handlers)| handlers)
            .collect_vec()
            .first()
            .cloned()
    }

    /// Get the handler associated with a given mime from mimeapps.list's default apps
    #[mutants::skip] // Cannot entirely test, namely cannot test selector functionality
    pub fn get_handler_from_user(
        &self,
        mime: &Mime,
        selector: &str,
        use_selector: bool,
    ) -> Result<DesktopHandler> {
        let error = Error::from(ErrorKind::NotFound(mime.to_string()));
        // Check for an exact match first and then fall back to wildcard
        match self
            .default_apps
            .get(mime)
            .or_else(|| self.get_from_wildcard(mime))
        {
            Some(handlers) if use_selector && handlers.len() > 1 => {
                let handlers = handlers
                    .iter()
                    .map(|h| Ok((h, h.get_entry()?.name)))
                    .collect::<Result<Vec<_>>>()?;

                let handler = {
                    let name =
                        select(selector, handlers.iter().map(|h| h.1.clone()))?;

                    handlers
                        .into_iter()
                        .find(|h| h.1 == name)
                        .ok_or(error)?
                        .0
                        .clone()
                };

                Ok(handler)
            }
            Some(handlers) => Ok(handlers.front().ok_or(error)?.clone()),
            None => Err(error),
        }
    }

    /// Get the path to the user's mimeapps.list file
    #[mutants::skip] // Cannot test directly, depends on system state
    fn path() -> Result<PathBuf> {
        let mut config = xdg::BaseDirectories::new()?.get_config_home();
        config.push("mimeapps.list");
        Ok(config)
    }

    /// Read and parse mimeapps.list
    #[mutants::skip] // Cannot test directly, depends on system state
    pub fn read() -> Result<Self> {
        let exists = std::path::Path::new(&Self::path()?).exists();

        let file = std::fs::OpenOptions::new()
            .write(!exists)
            .create(!exists)
            .read(true)
            .open(Self::path()?)?;

        Self::read_from(file)
    }

    /// Deserialize MimeApps from reader
    /// Makes testing easier
    fn read_from<R: Read>(reader: R) -> Result<Self> {
        let mut mime_apps: MimeApps = serde_ini::de::from_read(reader)?;

        // Remove empty entries
        mime_apps
            .default_apps
            .retain(|_, handlers| !handlers.is_empty());

        Ok(mime_apps)
    }

    /// Save associations to mimeapps.list
    #[mutants::skip] // Cannot test directly, alters system state
    pub fn save(&mut self) -> Result<()> {
        if cfg!(test) {
            Ok(())
        } else {
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .create(true)
                .write(true)
                .truncate(true)
                .open(Self::path()?)?;

            self.save_to(&mut file)
        }
    }

    /// Serialize MimeApps and write to writer
    /// Makes testing easier
    fn save_to<W: Write>(&mut self, writer: &mut W) -> Result<()> {
        // Remove empty entries
        self.default_apps.retain(|_, handlers| !handlers.is_empty());
        serde_ini::ser::to_writer(writer, self)?;
        Ok(())
    }
}

/// Run given selector command
#[mutants::skip] // Cannot test directly, runs external command
fn select<O: Iterator<Item = String>>(
    selector: &str,
    mut opts: O,
) -> Result<String> {
    use std::{
        io::prelude::*,
        process::{Command, Stdio},
    };

    let process = {
        let mut split = shlex::split(selector).ok_or_else(|| {
            Error::from(ErrorKind::BadCmd(selector.to_string()))
        })?;
        let (cmd, args) = (split.remove(0), split);
        Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?
    };

    let output = {
        process
            .stdin
            .ok_or_else(|| ErrorKind::Selector(selector.to_string()))?
            .write_all(opts.join("\n").as_bytes())?;

        let mut output = String::with_capacity(24);

        process
            .stdout
            .ok_or_else(|| ErrorKind::Selector(selector.to_string()))?
            .read_to_string(&mut output)?;

        output.trim_end().to_owned()
    };

    if output.is_empty() {
        Err(Error::from(ErrorKind::Cancelled))
    } else {
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{fs::File, str::FromStr};

    // Helper function to test serializing and deserializing mimeapps.list files
    fn mimeapps_round_trip(
        input_path: &str,
        expected_path: &str,
        mutation: fn(&mut MimeApps) -> Result<()>,
    ) -> Result<()> {
        let file = File::open(input_path)?;
        let mut mime_apps = MimeApps::read_from(file)?;

        mutation(&mut mime_apps)?;

        let mut buffer = Vec::new();
        mime_apps.save_to(&mut buffer)?;

        assert_eq!(
            String::from_utf8(buffer)?,
            // Unfortunately, serde_ini outputs \r\n line endings
            std::fs::read_to_string(expected_path)?.replace('\n', "\r\n")
        );

        Ok(())
    }

    // Helper function that does nothing
    fn noop(_: &mut MimeApps) -> Result<()> {
        Ok(())
    }

    // Helper function to reduce duplicate code for the most common case
    fn mimeapps_round_trip_simple(path: &str) -> Result<()> {
        mimeapps_round_trip(path, path, noop)
    }

    #[test]
    fn mimeapps_no_added_round_trip() -> Result<()> {
        mimeapps_round_trip_simple("./tests/mimeapps_no_added.list")
    }

    #[test]
    fn mimeapps_no_default_round_trip() -> Result<()> {
        mimeapps_round_trip_simple("./tests/mimeapps_no_default.list")
    }

    #[test]
    fn mimeapps_sorted_round_trip() -> Result<()> {
        mimeapps_round_trip_simple("./tests/mimeapps_sorted.list")
    }

    #[test]
    fn mimeapps_anomalous_semicolons_round_trip() -> Result<()> {
        mimeapps_round_trip(
            "./tests/mimeapps_anomalous_semicolons.list",
            "./tests/mimeapps_sorted.list",
            noop,
        )
    }

    #[test]
    fn mimeapps_empty_entry_round_trip() -> Result<()> {
        mimeapps_round_trip(
            "./tests/mimeapps_empty_entry.list",
            "./tests/mimeapps_no_added.list",
            noop,
        )
    }

    #[test]
    fn mimeapps_empty_entry_fallback() -> Result<()> {
        let file = File::open("./tests/mimeapps_empty_entry.list")?;
        let mime_apps = MimeApps::read_from(file)?;

        assert_eq!(
            mime_apps
                .get_handler_from_user(&mime::TEXT_PLAIN, "", false)?
                .to_string(),
            "nvim.desktop"
        );

        Ok(())
    }

    #[test]
    // This is mainly to check that "empty" entries don't get mixed in and complicate things
    fn mimeapps_round_trip_with_deletion_and_re_addition() -> Result<()> {
        let remove_and_re_add = |mime_apps: &mut MimeApps| {
            mime_apps.remove_handler(
                &mime::TEXT_HTML,
                &DesktopHandler::from_str("nvim.desktop")?,
            );
            mime_apps.add_handler(
                &mime::TEXT_HTML,
                &DesktopHandler::from_str("nvim.desktop")?,
            );
            Ok(())
        };

        let path = "./tests/mimeapps_sorted.list";

        mimeapps_round_trip(path, path, remove_and_re_add)
    }

    #[test]
    fn mimeapps_duplicate_round_trip() -> Result<()> {
        mimeapps_round_trip(
            "./tests/mimeapps_duplicate.list",
            "./tests/mimeapps_no_added.list",
            noop,
        )
    }
}
