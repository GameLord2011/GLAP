/*
    TODO:
        Fix color-to-tui or replace it.
*/

extern crate sdl2;

mod app;
mod audio;

use std::fs;

use crate::app::main::App;
use color_eyre::{Result, install};
use ratatui::run;
use sdl2::init;
use serde::{Serialize, Deserialize};

#[cfg(not(target_os = "macos"))]
#[used]
#[unsafe(link_section = ".text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

#[cfg(target_os = "macos")]
#[used]
#[unsafe(link_section = "__TEXT,__text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

#[derive(Serialize, Deserialize)]
pub struct Theme {
    #[serde(with = "color_to_tui")]
    foreground: ratatui::style::Color,
    #[serde(with = "color_to_tui")]
    background: ratatui::style::Color,
    #[serde(with = "color_to_tui")]
    controls: ratatui::style::Color,
    #[serde(with = "color_to_tui::optional")]
    playbar_played: Option<ratatui::style::Color>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    music_folder: Option<String>,
    i_definitely_know_the_controls: bool,
    theme: Option<Theme>
}

fn main() -> Result<()> {
    install()?;
    let config_dir = dirs::config_dir().unwrap().join("GLAP");
    let mut config: Option<Config> = None;
    // ~~idiomatic~~ idiotic Rust™
    if !fs::exists(&config_dir /* Refrences are a gift from God and shall be abused to their full potential. */)? {
        fs::create_dir(&config_dir)?;
        fs::write(config_dir.join("config.toml"), toml::to_string(&Config::default()).unwrap())?;
    } else {
        let d = fs::read_to_string(config_dir.join("config.toml"))?;
        config = Some(toml::from_str(&d).unwrap());
    }
    run(|terminal| App::default().run(terminal, init().unwrap().audio().unwrap(), config))?;
    Ok(())
}
