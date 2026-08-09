extern crate sdl2;

mod app;
mod audio;

use crate::app::main::App;
use color_eyre::{Result, install};
use ratatui::run;
use sdl2::init;

#[cfg(not(target_os = "macos"))]
#[used]
#[unsafe(link_section = ".text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

#[cfg(target_os = "macos")]
#[used]
#[unsafe(link_section = "__TEXT,__text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

fn main() -> Result<()> {
    install()?;
    run(|terminal| App::default().run(terminal, init().unwrap().audio().unwrap()))?;
    Ok(())
}
