use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub enable_selector: bool,
    pub selector: String,
    pub terminal_emulator: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enable_selector: false,
            selector: "rofi -dmenu -p 'Open With: '".to_owned(),
            terminal_emulator: "xterm -e".to_owned(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(confy::load("handlr")?)
    }

    pub fn select<O: Iterator<Item = String>>(
        &self,
        mut opts: O,
    ) -> Result<String> {
        use itertools::Itertools;
        use std::{
            io::prelude::*,
            process::{Command, Stdio},
        };

        let process = {
            let mut split = shlex::split(&self.selector).unwrap();
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
                .ok_or(Error::Selector(self.selector.clone()))?
                .write_all(opts.join("\n").as_bytes())?;

            let mut output = String::with_capacity(24);

            process
                .stdout
                .ok_or(Error::Selector(self.selector.clone()))?
                .read_to_string(&mut output)?;

            output.trim_end().to_owned()
        };

        if output.is_empty() {
            Err(Error::Cancelled)
        } else {
            Ok(output)
        }
    }
}
