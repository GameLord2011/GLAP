extern crate sdl2;

mod app;
mod audio;
mod utils;

use std::fs;

use crate::app::main::App;
use color_eyre::{Result, install};
use ratatui::run;
use sdl2::init;
use serde::{Deserialize, Serialize};

#[cfg(not(target_os = "macos"))]
#[used]
#[unsafe(link_section = ".text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

#[cfg(target_os = "macos")]
#[used]
#[unsafe(link_section = "__TEXT,__text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

#[derive(Serialize, Deserialize, Default)]
pub struct Theme {
    foreground: Option<String>,
    background: Option<String>,
    controls: Option<String>,
    playbar: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    music_folder: Option<String>,
    dont_filter_unplayable: Option<bool>,
    theme: Option<Theme>,
}

fn main() -> Result<()> {
    install()?;
    let config_dir = dirs::home_dir().unwrap().join(".config").join("GLAP");
    let mut config = Config::default();
    // ~~idiomatic~~ idiotic Rust™
    if fs::exists(
        &config_dir, /* Refrences are a gift from God and shall be abused to their full potential. */
    )? {
        let d = fs::read_to_string(config_dir.join("config.toml"))?;
        config = toml::from_str(&d).unwrap();
    } else {
        fs::create_dir_all(&config_dir)?;
        fs::write(
            config_dir.join("config.toml"),
            toml::to_string(&Config::default()).unwrap(),
        )?;
    }
    run(|terminal| App::default().run(terminal, init().unwrap().audio().unwrap(), config))?;
    Ok(())
}
