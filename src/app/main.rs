use std::{
    io::{self, Error},
    path::PathBuf,
    thread::JoinHandle,
    time::Duration,
};

use crossterm::event::{
    Event::{FocusLost, Key},
    KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Styled, Stylize},
    symbols,
    text::Line,
    widgets::{Block, BorderType, Borders, LineGauge, Paragraph, Widget, WidgetRef},
};
use ratatui_explorer::{FileExplorer, Theme};
use sdl2::{AudioSubsystem, audio::AudioDevice};
use tui_checkbox::Checkbox;

use crate::{
    Config,
    audio::{
        audio_player::AudioPlayer,
        processing::{create_device, process_samples_from_file},
    },
    utils::string_to_color,
};

const SUPPORTED_EXTENSIONS: [&str; 6] = ["mp2", "mp3", "flac", "ogg", "aac", "wav"];

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
    Settings1,
}

#[derive(Default, PartialEq)]
enum Page {
    #[default]
    Player,
    Settings,
}

/*
    "Erm acktually ChatGPT / Copilot / Claude / Deepseek / Librewhateverthef**kAIpeopleuse
    says that this is not idiomatic Rust and that the app state should be split into three
    different structs."

    NO. I will not do this. I am already in the 9'th circle of ownership hell so I don't
    want to add another thing to this. This is not a user-facing library so having a
    monolithic App struct is FINE. THIS IS MY FINAL SAY ON THIS DO NOT OPEN ANY PRS
    BUGGING ME ABOUT THIS.
*/
#[derive(Default)]
pub struct App {
    // App state
    page: Page,
    exit: bool,
    explorer_state: Option<FileExplorer>,
    focused: Focused,
    playing: bool,

    // Audio state.
    comment_string: String,
    helper_thread_handle: Option<JoinHandle<Result<AudioPlayer, Error>>>, // Peak type.
    audio_path: PathBuf,
    should_make_new_device: bool,
    current_device: Option<AudioDevice<AudioPlayer>>,
    audio_created: bool,
    audio_subsystem: Option<AudioSubsystem>,

    // playback state
    playback_ratio: Option<f64>,
    repeat_sate: RepeatState,
    queue: Vec<PathBuf>,
    que_idx: usize,

    // config
    fg: Color,
    bg: Color,
    playbar: Color,
    controls: Color,
    dont_filter_unplayable: bool,
}

impl App {
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        audio_subsystem: AudioSubsystem,
        config: Config,
    ) -> io::Result<()> {
        // Explorer creation.
        self.audio_subsystem = Some(audio_subsystem);

        self.explorer_state = Some(FileExplorer::new().unwrap());
        self.explorer_state.as_mut().unwrap().set_theme(
            Theme::new().with_highlight_symbol("> ").with_block(
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_type(BorderType::Double),
            ),
        );

        if !config.dont_filter_unplayable.unwrap_or_default() {
            self.dont_filter_unplayable = true;
            self.explorer_state.as_mut().unwrap().set_filter_map(|f| {
                if match f.path.extension() {
                    Some(e) => {
                        let extension = e.to_str().unwrap_or_default();
                        SUPPORTED_EXTENSIONS.contains(&extension)
                    }
                    None => f.is_dir,
                } {
                    Some(f)
                } else {
                    None
                }
            })?;
        }

        if let Some(d) = config.music_folder {
            self.explorer_state.as_mut().unwrap().set_cwd(d)?;
        } else if let Some(d) = dirs::audio_dir() {
            self.explorer_state.as_mut().unwrap().set_cwd(d)?;
        }

        if let Some(theme) = config.theme {
            if let Some(fg) = theme.foreground {
                self.fg = string_to_color(fg);
            }
            if let Some(bg) = theme.background {
                self.bg = string_to_color(bg);
            }
            if let Some(pb) = theme.playbar {
                self.playbar = string_to_color(pb);
            } else {
                self.playbar = Color::Red;
            }
            if let Some(controls) = theme.controls {
                self.controls = string_to_color(controls);
            } else {
                self.controls = Color::Green
            }
        } else {
            self.playbar = Color::Red;
            self.controls = Color::Green;
        }

        loop {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            if self.exit {
                break;
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // Device creation. I put it here because it can take some time and requires querying :P
        if self.should_make_new_device {
            if self.helper_thread_handle.is_none() {
                #[cfg(debug_assertions)]
                {
                    self.comment_string = "Parsing audio samples; this may take a while in a development environment.".to_owned();
                }
                self.helper_thread_handle = Some(process_samples_from_file(
                    self.audio_path.clone().to_str().unwrap().to_owned(),
                ));
            } else {
                if self.helper_thread_handle.as_ref().unwrap().is_finished() {
                    let d = self.helper_thread_handle.take().unwrap().join().unwrap();
                    if let Err(e) = d {
                        self.comment_string =
                            format!("Error on file {}: {}", self.audio_path.to_str().unwrap(), e)
                                .to_string()
                    } else {
                        self.comment_string = self
                            .audio_path
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned();
                        self.current_device = Some(create_device(
                            d.unwrap(),
                            self.audio_subsystem.as_ref().unwrap().clone(),
                        ));
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
                    }
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
                            Focused::Playbar if self.audio_created => {
                                self.current_device.as_mut().unwrap().lock().back_2s()
                            }
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
                            Focused::Playbar if self.audio_created => {
                                self.current_device.as_mut().unwrap().lock().forward_2s()
                            }
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

                            Focused::Back => {
                                self.current_device.as_mut().unwrap().lock().restart();
                                self.playing = true;
                            }

                            Focused::Forward => match self.repeat_sate {
                                RepeatState::None => (),
                                RepeatState::RepeatOne => {
                                    self.current_device.as_mut().unwrap().lock().restart();
                                }
                                RepeatState::RepeatAll => {
                                    self.current_device.as_mut().unwrap().lock().finished = true;
                                }
                            },

                            Focused::Repeat => match self.repeat_sate {
                                RepeatState::None => {
                                    self.repeat_sate = RepeatState::RepeatAll;
                                    self.queue = self
                                        .explorer_state
                                        .as_ref()
                                        .unwrap()
                                        .files()
                                        .iter()
                                        .map(|f| f.path.clone())
                                        .collect();
                                    self.queue.remove(0); // On most systems this is the ../ directory.
                                    let idx = self.queue.iter().position(|n| n == &self.audio_path);
                                    if let Some(i) = idx {
                                        self.queue.rotate_left(i - 1);
                                    }
                                    self.que_idx = 0;
                                    // self.should_make_new_device = true;
                                }
                                RepeatState::RepeatAll => self.repeat_sate = RepeatState::RepeatOne,
                                RepeatState::RepeatOne => self.repeat_sate = RepeatState::None,
                            },

                            _ => (),
                        },

                        Key(KeyEvent {
                            code: KeyCode::Char('s'),
                            modifiers: KeyModifiers::CONTROL,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }) => {
                            self.page = Page::Settings;
                            self.focused = Focused::Settings1;
                        }

                        FocusLost => self.focused = Focused::None,

                        _ => (),
                    }

                    if self.focused == Focused::Explorer {
                        self.explorer_state.as_mut().unwrap().handle(&event)?;
                    }
                }

                Page::Settings => match event {
                    Key(KeyEvent {
                        code: KeyCode::Char('p'),
                        modifiers: KeyModifiers::CONTROL,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => {
                        self.page = Page::Player;
                        self.focused = Focused::Play;
                    },
                    Key(KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => {
                        match self.focused {
                            Focused::Settings1 => {
                                if self.dont_filter_unplayable {
                                    self.dont_filter_unplayable = false;
                                    self.explorer_state.as_mut().unwrap().set_filter_map(|x| Some(x))?;
                                } else {
                                    self.dont_filter_unplayable = true;
                                    self.explorer_state.as_mut().unwrap().set_filter_map(|f| {
                                        if match f.path.extension() {
                                            Some(e) => {
                                                let extension = e.to_str().unwrap_or_default();
                                                SUPPORTED_EXTENSIONS.contains(&extension)
                                            }
                                            None => f.is_dir,
                                        } {
                                            Some(f)
                                        } else {
                                            None
                                        }
                                    })?;
                                }
                            },
                            _ => ()
                        }
                    },
                    _ => (),
                },
            }
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::bordered()
            .border_type(BorderType::Double)
            .style(Style::default().fg(self.fg).bg(self.bg));
        let highlighted = match (self.bg == Color::Reset, self.fg == Color::Reset) {
            (false, false) => Style::new().fg(self.bg).bg(self.fg),
            _ => Style::new().fg(Color::Black).bg(Color::White),
        };

        match self.page {
            Page::Player => {
                if self.focused == Focused::Explorer {
                    block = block.title_bottom(Line::from(vec![
                        " Quit ".into(),
                        "^q".fg(self.controls).bold(),
                        "; Settings ".into(),
                        "^s".fg(self.controls).bold(),
                        "; Close Explorer ".into(),
                        "[TAB] ".fg(self.controls).bold(),
                    ]));
                } else {
                    block = block.title_bottom(Line::from(vec![
                        " Quit ".into(),
                        "^q".fg(self.controls).bold(),
                        "; Settings ".into(),
                        "^s".fg(self.controls).bold(),
                        "; Open Explorer ".into(),
                        "[TAB] ".fg(self.controls).bold(),
                    ]));
                }
                block = block.title(Line::from(" GLAP | Player ").left_aligned());

                let playback_area;
                if self.focused == Focused::Explorer {
                    let layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Ratio(1, 4),
                            Constraint::Length(1),
                            Constraint::Fill(1),
                        ])
                        .split(block.inner(area));

                    playback_area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(50), Constraint::Fill(1)])
                        .split(layout[2]);

                    let explorer = self.explorer_state.as_ref().unwrap().widget();
                    explorer.render_ref(layout[0], buf);
                } else {
                    playback_area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(50), Constraint::Fill(1)])
                        .split(block.inner(area));
                }

                block.render(area, buf);

                let mut back = Paragraph::new("\u{F04AB}");
                let mut play_button = Paragraph::new(match self.playing {
                    true => "\u{F03E4}",
                    false => "\u{F040A}",
                });
                let mut forward = Paragraph::new("\u{F04AC}");
                let mut repeat = Paragraph::new(match self.repeat_sate {
                    RepeatState::None => "\u{F0457}",
                    RepeatState::RepeatAll => "\u{F0456}",
                    RepeatState::RepeatOne => "\u{F0458}",
                });
                if self.repeat_sate == RepeatState::None {
                    forward = forward.dim()
                }

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
                    .filled_style(Style::new().fg(self.playbar))
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

                if self.audio_created
                    && let Some(r) = self.playback_ratio
                {
                    progress_bar = progress_bar.ratio(r);
                } else {
                    progress_bar = progress_bar.ratio(0_f64);
                }

                Paragraph::new(self.comment_string.clone()).render(playback_area[0], buf);
                progress_bar.render(playbar_area[0], buf);
                back.render(controls_area[1], buf);
                play_button.render(controls_area[3], buf);
                forward.render(controls_area[5], buf);
                repeat.render(controls_area[7], buf);
            }

            Page::Settings => {
                block = block.title_bottom(Line::from(vec![
                    " Quit ".into(),
                    "^q".fg(self.controls).bold(),
                    "; Player ".into(),
                    "^p ".fg(self.controls).bold(),
                ]));

                let settings_layout = Layout::default()
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Fill(1),
                    ])
                    .split(block.inner(area));

                block.render(area, buf);
                let mut filter_unplayable = Checkbox::new("Filter unplayable files?", self.dont_filter_unplayable)
                    .checked_symbol("[X]")
                    .unchecked_symbol("[ ]");

                match self.focused {
                    Focused::Settings1 => filter_unplayable = filter_unplayable.set_style(highlighted),
                    _ => (),
                }

                filter_unplayable.render(settings_layout[1], buf);
            }
        }
    }
}
