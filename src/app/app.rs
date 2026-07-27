use std::{io, path::PathBuf, time::Duration};

use crossterm::event::{Event::Key, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, BorderType, Borders, LineGauge, Paragraph, Widget, WidgetRef, Wrap},
};
use ratatui_explorer::{FileExplorer, Theme};
use sdl2::{AudioSubsystem, audio::AudioDevice};

use crate::audio::{self, audio_player::AudioPlayer, player::create_device};

#[derive(Default, PartialEq)]
enum Page {
    About,
    #[default]
    Player,
}

#[derive(Default)]
pub struct App {
    page: Page,
    helper_thread_handle: Option<std::thread::JoinHandle<AudioPlayer>>,
    audio_path: String,
    should_make_new_device: bool,
    current_device: Option<AudioDevice<AudioPlayer>>,
    audio_created: bool,
    exit: bool,
    explorer_state: Option<FileExplorer>,
    explorer_focused: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            explorer_focused: true,
            ..Default::default()
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        audio_subsystem: AudioSubsystem,
    ) -> io::Result<()> {
        self.explorer_state = Some(FileExplorer::new().unwrap());
        self.explorer_state.as_mut().unwrap().set_theme(
            Theme::new().with_highlight_symbol("> ").with_block(
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_type(BorderType::Double),
            ),
        );
        let audio_dir = dirs::audio_dir();
        if audio_dir.is_some() {
            self.explorer_state
                .as_mut()
                .unwrap()
                .set_cwd(audio_dir.unwrap())?;
        }
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
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self, audio_subsystem: &AudioSubsystem) -> io::Result<()> {
        if !self.audio_created && self.should_make_new_device {
            if self.helper_thread_handle.is_none() {
                self.helper_thread_handle = Some(audio::player::process_samples_from_file(
                    self.audio_path.clone(),
                    1_f32,
                ));
            } else {
                if self.helper_thread_handle.as_ref().unwrap().is_finished() {
                    let d = self.helper_thread_handle.take().unwrap().join().unwrap();
                    self.current_device = Some(create_device(d, audio_subsystem.clone()).1);
                    self.current_device.as_mut().unwrap().resume();
                    self.audio_created = true;
                    self.should_make_new_device = false;
                }
            }
        }
        if crossterm::event::poll(Duration::from_millis(50))? {
            let event = crossterm::event::read()?;
            if event
                == Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                })
            {
                self.exit = true;
            }
            match self.page {
                Page::About => {
                    if event
                        == Key(KeyEvent {
                            code: KeyCode::Char('p'),
                            modifiers: KeyModifiers::CONTROL,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        })
                    {
                        self.page = Page::Player
                    }
                }
                Page::Player => {
                    if event
                        == Key(KeyEvent {
                            code: KeyCode::Tab,
                            modifiers: KeyModifiers::NONE,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        })
                    {
                        self.explorer_focused = !self.explorer_focused;
                    }
                    if self.explorer_focused && event == Key(KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) {
                        self.audio_path = self.explorer_state.as_mut().unwrap().current().path.clone().to_str().unwrap().to_owned();
                        self.should_make_new_device = true;
                    }
                    if self.explorer_focused {
                        self.explorer_state.as_mut().unwrap().handle(&event)?;
                    }
                    if event
                        == Key(KeyEvent {
                            code: KeyCode::Char('a'),
                            modifiers: KeyModifiers::CONTROL,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        })
                    {
                        self.page = Page::About
                    }
                }
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::bordered().border_type(BorderType::Double);

        match self.page {
            Page::Player => {
                block = block
                    .title_bottom(Line::from(Line::from(vec![
                        " Quit ".into(),
                        "<CTRL + Q>".green().bold(),
                        " About ".into(),
                        "<CTRL + A>".green().bold(),
                        " Toggle Explorer Focus ".into(),
                        "<TAB> ".green().bold(),
                    ])))
                    .title(Line::from(" GLAP | Player ").left_aligned());
                let inner_area = block.inner(area);
                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Ratio(1, 4),
                        Constraint::Length(1),
                        Constraint::Fill(1),
                    ])
                    .split(inner_area);
                let left = layout[0];
                let right = layout[2];
                block.render(area, buf);
                let explorer = self.explorer_state.as_ref().unwrap().widget();
                explorer.render_ref(left, buf);
                let progress_bar = LineGauge::default()
                    .filled_style(Style::new().red().bold())
                    .label("Played")
                    .ratio(0_f64)
                    .filled_symbol(symbols::line::THICK_HORIZONTAL)
                    .unfilled_symbol(symbols::line::THICK_HORIZONTAL);
                progress_bar.render(right, buf);
            }

            Page::About => {
                block = block
                    .title_bottom(Line::from(Line::from(vec![
                        " Quit ".into(),
                        "<CTRL + Q>".green().bold(),
                        " Player ".into(),
                        "<CTRL + P> ".green().bold(),
                    ])))
                    .title(Line::from(" GLAP | About ").left_aligned());
                let about_text = include_str!("about.txt");
                Paragraph::new(about_text)
                    .left_aligned()
                    .wrap(Wrap { trim: false })
                    .block(block)
                    .render(area, buf);
            }
        }
    }
}
