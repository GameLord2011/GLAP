extern crate sdl2;

use crate::app::main::App;
use color_eyre::{Result, install};
use ratatui::run;
use sdl2::init;

mod app;
mod audio;

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
    let audio_subsystem = init().unwrap().audio().unwrap();
    run(|terminal| App::default().run(terminal, audio_subsystem))?;
    Ok(())
}
