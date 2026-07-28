use std::{
    io::{self, Error},
    path::PathBuf,
    time::Duration,
};

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

use crate::audio::{
    audio_player::AudioPlayer, player::create_device, player::process_samples_from_file,
};

#[derive(Default, PartialEq)]
enum Page {
    About,
    #[default]
    Player,
}

#[derive(Default)]
pub struct App {
    page: Page,
    comment_string: String,
    helper_thread_handle: Option<std::thread::JoinHandle<Result<AudioPlayer, Error>>>,
    audio_path: PathBuf,
    should_make_new_device: bool,
    current_device: Option<AudioDevice<AudioPlayer>>,
    audio_created: bool,
    exit: bool,
    explorer_state: Option<FileExplorer>,
    explorer_open: bool,
    playback_ratio: Option<f64>,
}

impl App {
    pub fn new() -> Self {
        Self {
            explorer_open: true,
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
        if self.should_make_new_device {
            if self.helper_thread_handle.is_none() {
                self.helper_thread_handle = Some(process_samples_from_file(
                    self.audio_path.clone().to_str().unwrap().to_owned(),
                    1_f32,
                ));
            } else {
                if self.helper_thread_handle.as_ref().unwrap().is_finished() {
                    let d = self.helper_thread_handle.take().unwrap().join().unwrap();
                    if d.is_err() {
                        self.comment_string = format!(
                            "Error on file {}: {}",
                            self.audio_path.to_str().unwrap(),
                            d.unwrap_err()
                        )
                        .to_string()
                    } else {
                        self.comment_string = self
                            .audio_path
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned();
                        self.current_device =
                            Some(create_device(d.unwrap(), audio_subsystem.clone()).1);
                        self.current_device.as_mut().unwrap().resume();
                        self.audio_created = true;
                    }
                    self.should_make_new_device = false;
                }
            }
        }
        if self.audio_created {
            self.playback_ratio = Some(self.current_device.as_mut().unwrap().lock().get_progress());
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
                        self.explorer_open = !self.explorer_open;
                    }
                    if self.explorer_open
                        && event
                            == Key(KeyEvent {
                                code: KeyCode::Enter,
                                modifiers: KeyModifiers::NONE,
                                kind: KeyEventKind::Press,
                                state: KeyEventState::NONE,
                            })
                        && self.explorer_state.as_ref().unwrap().current().is_file()
                    {
                        self.audio_path = self
                            .explorer_state
                            .as_ref()
                            .unwrap()
                            .current()
                            .path
                            .to_owned();
                        self.should_make_new_device = true;
                    }
                    if self.explorer_open {
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
                        "; About ".into(),
                        "<CTRL + A>".green().bold(),
                        "; Toggle Explorer Open ".into(),
                        "<TAB> ".green().bold(),
                    ])))
                    .title(Line::from(" GLAP | Player ").left_aligned());
                let inner_area = block.inner(area);
                let playback_area;
                if self.explorer_open {
                    let layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Ratio(1, 4),
                            Constraint::Length(1),
                            Constraint::Fill(1),
                        ])
                        .split(inner_area);
                    let left = layout[0];
                    playback_area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(50), Constraint::Fill(1)])
                        .split(layout[2]);
                    let explorer = self.explorer_state.as_ref().unwrap().widget();
                    explorer.render_ref(left, buf);
                } else {
                    playback_area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(50), Constraint::Fill(1)])
                        .split(block.inner(area));
                }
                block.render(area, buf);
                let mut progress_bar = LineGauge::default()
                    .filled_style(Style::new().red().bold())
                    .label("Played")
                    .filled_symbol(symbols::line::THICK_HORIZONTAL)
                    .unfilled_symbol(symbols::line::THICK_HORIZONTAL);
                if self.audio_created && self.playback_ratio.is_some() {
                    progress_bar = progress_bar.ratio(self.playback_ratio.unwrap());
                } else {
                    progress_bar = progress_bar.ratio(0_f64);
                }
                let title = Paragraph::new(self.comment_string.clone());
                title.render(playback_area[0], buf);
                progress_bar.render(playback_area[1], buf);
            }

            Page::About => {
                block = block
                    .title_bottom(Line::from(Line::from(vec![
                        " Quit ".into(),
                        "<CTRL + Q>".green().bold(),
                        "; Player ".into(),
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
