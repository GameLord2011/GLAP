use std::io;

use crossterm::event::{Event::Key, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    symbols::{self, border},
    text::Line,
    widgets::{Block, LineGauge, Paragraph, Wrap},
};
use sdl2::{AudioSubsystem, audio::AudioDevice};

use crate::audio::{self, audio_player::AudioPlayer};

#[derive(Default)]
enum Page {
    About,
    #[default]
    Player,
}

#[derive(Default)]
pub struct App {
    page: Page,
    current_device: Option<AudioDevice<AudioPlayer>>,
    audio_created: bool,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal, audio_subsystem: AudioSubsystem) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&audio_subsystem)?;
            if self.exit {
                break;
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let mut block = Block::bordered().border_set(border::DOUBLE);
        match self.page {
            Page::About => {
                block = block
                    .title(Line::from(" GLAP | About ").left_aligned())
                    .title_bottom(Line::from(vec![
                        " Quit ".into(),
                        "<CTRL + Q>".green().bold(),
                        " Player ".into(),
                        "<CTRL + P> ".green().bold(),
                    ]));
                let inner_area = block.inner(frame.area());
                let about_para =
                    Paragraph::new(include_str!("about.txt")).wrap(Wrap { trim: false });
                frame.render_widget(block, frame.area());
                // frame.render_widget(RatatuiLogo::small(), inner_area);
                frame.render_widget(about_para, inner_area);
            }
            Page::Player => {
                let layout =
                    Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).spacing(1);
                let [_top, bottom] = frame.area().layout(&layout);
                block = block
                    .title(Line::from(" GLAP | Player ").left_aligned())
                    .title_bottom(Line::from(vec![
                        " Quit ".into(),
                        "<CTRL + Q>".green().bold(),
                        " About ".into(),
                        "<CTRL + A> ".green().bold(),
                    ]));
                let line_guage = LineGauge::default()
                    .filled_style(Style::new().red().bold())
                    .label("Played")
                    .ratio(0.5)
                    .filled_symbol(symbols::line::THICK_HORIZONTAL)
                    .unfilled_symbol(symbols::line::THICK_HORIZONTAL);
                frame.render_widget(block, frame.area());
                frame.render_widget(line_guage, bottom);
            }
        }
    }

    fn handle_events(&mut self, audio_subsystem: &AudioSubsystem) -> io::Result<()> {
        match crossterm::event::read()? {
            Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.page = Page::About;
                if !self.audio_created {
                    let player = audio::player::process_samples_from_file(dirs::audio_dir().unwrap().to_str().unwrap().to_owned() + "/sans.ogg", 1_f32);
                    self.current_device = Some(audio::player::create_device(player, audio_subsystem.clone()).1);
                    self.current_device.as_mut().unwrap().resume();
                    self.audio_created = true;
                }
            },
            Key(KeyEvent {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.page = Page::Player,
            Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.exit = true,
            _ => (),
        }
        Ok(())
    }
}
