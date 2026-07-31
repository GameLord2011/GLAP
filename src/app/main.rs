use std::{
    io::{self, Error},
    path::PathBuf,
    time::Duration,
    thread::JoinHandle
};

use crossterm::event::{Event::Key, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols,
    text::Line,
    widgets::{Block, BorderType, Borders, LineGauge, Paragraph, Widget, WidgetRef, Wrap},
};
use ratatui_explorer::{FileExplorer, Theme};
use sdl2::{AudioSubsystem, audio::AudioDevice};

use crate::audio::{
    audio_player::AudioPlayer,
    processing::{create_device, process_samples_from_file},
};

#[derive(Default, PartialEq)]
enum RepeatState {
    #[default]
    None,
    RepeatAll,
    RepeatOne,
}

#[derive(Default, PartialEq)]
enum Focused {
    #[default]
    Explorer,
    None,
    Back,
    Play,
    Forward,
    Repeat,
    Playbar,
}

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
    helper_thread_handle: Option<JoinHandle<Result<AudioPlayer, Error>>>,
    audio_path: PathBuf,
    should_make_new_device: bool,
    current_device: Option<AudioDevice<AudioPlayer>>,
    audio_created: bool,
    exit: bool,
    explorer_state: Option<FileExplorer>,
    playback_ratio: Option<f64>,
    focused: Focused,
    playing: bool,
    repeat_sate: RepeatState,
    queue: Vec<PathBuf>,
    que_idx: usize,
}

impl App {
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        audio_subsystem: AudioSubsystem,
    ) -> io::Result<()> {
        // Explorer creation.
        self.explorer_state = Some(FileExplorer::new().unwrap());
        self.explorer_state.as_mut().unwrap().set_theme(
            Theme::new().with_highlight_symbol("> ").with_block(
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_type(BorderType::Double),
            ),
        );
        let audio_dir = dirs::audio_dir();
        if let Some(d) = audio_dir {
            self.explorer_state
                .as_mut()
                .unwrap()
                .set_cwd(d)?;
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
        // Device creation. I put it here because it can take some time :P
        if self.should_make_new_device {
            if self.helper_thread_handle.is_none() {
                #[cfg(debug_assertions)]
                {
                    self.comment_string = "Parsing audio samples; this may take a while in a development environment.".to_owned();
                }
                self.helper_thread_handle = Some(process_samples_from_file(
                    self.audio_path.clone().to_str().unwrap().to_owned()
                ));
            } else {
                if self.helper_thread_handle.as_ref().unwrap().is_finished() {
                    let d = self.helper_thread_handle.take().unwrap().join().unwrap();
                    if let Err(e) = d {
                        self.comment_string = format!(
                            "Error on file {}: {}",
                            self.audio_path.to_str().unwrap(),
                            e
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
                        self.playing = true;
                    }
                    self.should_make_new_device = false;
                }
            }
        }

        // Audio handling stuff.
        if self.audio_created {
            self.playback_ratio = Some(self.current_device.as_mut().unwrap().lock().get_progress());
            if self.current_device.as_mut().unwrap().lock().finished {
                match self.repeat_sate {
                    RepeatState::None => self.current_device.as_mut().unwrap().pause(),
                    RepeatState::RepeatAll => {
                        if self.que_idx == self.queue.len() {
                            self.que_idx = 0;
                            self.audio_path = self.queue[0].clone();
                        } else {
                            self.audio_path = self.queue[self.que_idx].clone();
                        }
                        self.que_idx += 1;
                        self.should_make_new_device = true;
                        self.audio_created = false;
                    },
                    RepeatState::RepeatOne => {
                        self.current_device.as_mut().unwrap().lock().restart()
                    }
                }
            }
        }

        if crossterm::event::poll(Duration::ZERO)? {
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
                    match event {
                        Key(KeyEvent {
                            code: KeyCode::Char(' '),
                            modifiers: KeyModifiers::NONE,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }) => {
                            if self.audio_created {
                                if self.playing {
                                    self.current_device.as_mut().unwrap().pause();
                                    self.playing = false;
                                } else {
                                    self.current_device.as_mut().unwrap().resume();
                                    self.playing = true;
                                }
                            }
                        }

                        Key(KeyEvent {
                            code: KeyCode::Tab,
                            modifiers: KeyModifiers::NONE,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }) => {
                            if self.focused == Focused::Explorer {
                                self.focused = Focused::None
                            } else {
                                self.focused = Focused::Explorer
                            }
                        }

                        Key(KeyEvent {
                            code: KeyCode::Up,
                            modifiers: KeyModifiers::NONE,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }) => match self.focused {
                            Focused::Playbar => (),
                            Focused::Explorer => (),
                            _ => self.focused = Focused::Playbar,
                        },

                        Key(KeyEvent {
                            code: KeyCode::Down,
                            modifiers: KeyModifiers::NONE,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }) => match self.focused {
                            Focused::Playbar => self.focused = Focused::Play,
                            Focused::Explorer => (),
                            _ => (),
                        },

                        Key(KeyEvent {
                            code: KeyCode::Left,
                            modifiers: KeyModifiers::NONE,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }) => match self.focused {
                            Focused::Play => self.focused = Focused::Back,
                            Focused::Forward => self.focused = Focused::Play,
                            Focused::Repeat => self.focused = Focused::Forward,
                            Focused::None => self.focused = Focused::Back,
                            _ => (),
                        },

                        Key(KeyEvent {
                            code: KeyCode::Right,
                            modifiers: KeyModifiers::NONE,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }) => match self.focused {
                            Focused::Back => self.focused = Focused::Play,
                            Focused::Play => self.focused = Focused::Forward,
                            Focused::Forward => self.focused = Focused::Repeat,
                            Focused::None => self.focused = Focused::Forward,
                            _ => (),
                        },

                        Key(KeyEvent {
                            code: KeyCode::Enter,
                            modifiers: KeyModifiers::NONE,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }) => match self.focused {
                            Focused::Explorer => {
                                if self.explorer_state.as_ref().unwrap().current().is_file() {
                                    self.audio_path = self
                                        .explorer_state
                                        .as_ref()
                                        .unwrap()
                                        .current()
                                        .path
                                        .to_owned();
                                    self.should_make_new_device = true;
                                }
                            }

                            Focused::Play => {
                                if self.audio_created {
                                    if self.playing {
                                        self.current_device.as_mut().unwrap().pause();
                                        self.playing = false;
                                    } else {
                                        self.current_device.as_mut().unwrap().resume();
                                        self.playing = true;
                                    }
                                }
                            }

                            Focused::Back => self.current_device.as_mut().unwrap().lock().restart(),

                            Focused::Forward => match self.repeat_sate {
                                RepeatState::None => (),
                                RepeatState::RepeatOne => {
                                    self.current_device.as_mut().unwrap().lock().restart();
                                }
                                RepeatState::RepeatAll => {
                                    self.current_device.as_mut().unwrap().lock().finished = true;
                                },
                            },

                            Focused::Repeat => match self.repeat_sate {
                                RepeatState::None => {
                                    self.repeat_sate = RepeatState::RepeatAll;
                                    self.queue = self.explorer_state
                                        .as_ref()
                                        .unwrap()
                                        .files()
                                        .iter()
                                        .map(|f| f.path.clone())
                                        .collect();
                                    self.queue.remove(0); // On most systems this is the ../ directory.
                                    let idx = self.queue.iter().position(|n| n == &self.audio_path);
                                    if idx.is_some() {
                                        self.queue.rotate_left(idx.unwrap() + 1);
                                    }
                                    self.que_idx = 0;
                                    // self.should_make_new_device = true;
                                },
                                RepeatState::RepeatAll => self.repeat_sate = RepeatState::RepeatOne,
                                RepeatState::RepeatOne => self.repeat_sate = RepeatState::None,
                            },

                            _ => (),
                        },

                        Key(KeyEvent {
                            code: KeyCode::Char('a'),
                            modifiers: KeyModifiers::CONTROL,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }) => self.page = Page::About,

                        _ => (),
                    }

                    if self.focused == Focused::Explorer {
                        self.explorer_state.as_mut().unwrap().handle(&event)?;
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
        let highlighted = Style::default()
            .fg(ratatui::style::Color::Black)
            .bg(ratatui::style::Color::White);

        match self.page {
            Page::Player => {
                if self.focused == Focused::Explorer {
                    block = block.title_bottom(Line::from(vec![
                        " Quit ".into(),
                        "<CTRL + Q>".green().bold(),
                        "; About ".into(),
                        "<CTRL + A>".green().bold(),
                        "; Close Explorer ".into(),
                        "<TAB> ".green().bold(),
                    ]));
                } else {
                    block = block.title_bottom(Line::from(vec![
                        " Quit ".into(),
                        "<CTRL + Q>".green().bold(),
                        "; About ".into(),
                        "<CTRL + A>".green().bold(),
                        "; Open Explorer ".into(),
                        "<TAB> ".green().bold(),
                    ]));
                }
                block = block.title(Line::from(" GLAP | Player ").left_aligned());

                let inner_area = block.inner(area);
                let playback_area;
                if self.focused == Focused::Explorer {
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

                let mut play_button = Paragraph::new(
                    match self.playing {
                        true => "\u{F03E4}",
                        false => "\u{F040A}"
                    }
                );
                let mut repeat = Paragraph::new(
                    match self.repeat_sate {
                        RepeatState::None => "\u{F0457}",
                        RepeatState::RepeatAll => "\u{F0456}",
                        RepeatState::RepeatOne => "\u{F0458}"
                    }
                );
                let mut back = Paragraph::new("\u{F04AB}");
                let mut forward = Paragraph::new("\u{F04AC}");

                let playbar_area = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Fill(1),
                    ])
                    .split(playback_area[1]);
                let controls_area = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Fill(1),
                        /* Index 1; Back */ Constraint::Length(1),
                        Constraint::Length(1),
                        /* Index 3; Play / Pause */ Constraint::Length(1),
                        Constraint::Length(1),
                        /* Index 5; Forward */ Constraint::Length(1),
                        Constraint::Length(1),
                        /* Index 7; Repeat */ Constraint::Length(1),
                        Constraint::Fill(1),
                    ])
                    .split(playbar_area[2]);

                let mut progress_bar = LineGauge::default()
                    .filled_style(Style::new().red().bold())
                    .label("Played")
                    .filled_symbol(symbols::line::THICK_HORIZONTAL)
                    .unfilled_symbol(symbols::line::THICK_HORIZONTAL);

                match self.focused {
                    Focused::Back => {
                        back = back.set_style(highlighted);
                    }
                    Focused::Play => {
                        play_button = play_button.set_style(highlighted);
                    }
                    Focused::Forward => {
                        forward = forward.set_style(highlighted);
                    }
                    Focused::Repeat => {
                        repeat = repeat.set_style(highlighted);
                    }
                    Focused::Playbar => {
                        progress_bar = progress_bar.set_style(highlighted);
                    }
                    _ => (),
                }

                if self.audio_created && self.playback_ratio.is_some() {
                    progress_bar = progress_bar.ratio(self.playback_ratio.unwrap());
                } else {
                    progress_bar = progress_bar.ratio(0_f64);
                }

                let title = Paragraph::new(self.comment_string.clone());
                title.render(playback_area[0], buf);
                progress_bar.render(playbar_area[0], buf);
                back.render(controls_area[1], buf);
                play_button.render(controls_area[3], buf);
                forward.render(controls_area[5], buf);
                repeat.render(controls_area[7], buf);
            }

            Page::About => {
                block = block
                    .title_bottom(Line::from(vec![
                        " Quit ".into(),
                        "<CTRL + Q>".green().bold(),
                        "; Player ".into(),
                        "<CTRL + P> ".green().bold(),
                    ]))
                    .title(Line::from(" GLAP | About ").left_aligned());
                Paragraph::new(include_str!("about.txt"))
                    .left_aligned()
                    .wrap(Wrap { trim: false })
                    .block(block)
                    .render(area, buf);
            }
        }
    }
}
