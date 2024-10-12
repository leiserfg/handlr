mod apps;
mod cli;
mod common;
mod config;
mod error;
mod utils;

use apps::SystemApps;
use cli::Cmd;
use common::mime_table;
use config::Config;
use error::Result;

use clap::Parser;

#[mutants::skip] // Cannot test directly at the moment
fn main() -> Result<()> {
    let mut config = Config::new()?;
    let mut stdout = std::io::stdout().lock();

    let res = match Cmd::parse() {
        Cmd::Set { mime, handler } => config.set_handler(&mime, &handler),
        Cmd::Add { mime, handler } => config.add_handler(&mime, &handler),
        Cmd::Launch {
            mime,
            args,
            selector_args,
        } => {
            config.override_selector(selector_args);
            config.launch_handler(&mime, args)
        }
        Cmd::Get {
            mime,
            json,
            selector_args,
        } => {
            config.override_selector(selector_args);
            config.show_handler(&mut stdout, &mime, json)
        }
        Cmd::Open {
            paths,
            selector_args,
        } => {
            config.override_selector(selector_args);
            config.open_paths(&paths)
        }
        Cmd::Mime { paths, json } => {
            mime_table(&mut stdout, &paths, json, config.terminal_output)
        }
        Cmd::List { all, json } => config.print(&mut stdout, all, json),
        Cmd::Unset { mime } => config.unset_handler(&mime),
        Cmd::Remove { mime, handler } => config.remove_handler(&mime, &handler),
        Cmd::Autocomplete {
            desktop_files,
            mimes,
        } => {
            if desktop_files {
                SystemApps::list_handlers(&mut stdout)?;
            } else if mimes {
                common::db_autocomplete(&mut stdout)?;
            }
            Ok(())
        }
    };

    // Issue a notification if handlr is not being run in a terminal
    if let Err(ref e) = res {
        if !config.terminal_output {
            utils::notify("handlr error", &e.to_string())?
        }
    }

    res
}
