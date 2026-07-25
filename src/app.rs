use std::io;

use crossterm::event::{Event::Key, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Wrap},
};

#[derive(Default)]
enum Page {
    About,
    #[default]
    Player,
}

#[derive(Default)]
pub struct App {
    page: Page,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
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
                block = block
                    .title(Line::from(" GLAP | Player ").left_aligned())
                    .title_bottom(Line::from(vec![
                        " Quit ".into(),
                        "<CTRL + Q>".green().bold(),
                        " About ".into(),
                        "<CTRL + A> ".green().bold(),
                    ]));
                frame.render_widget(block, frame.area())
            }
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match crossterm::event::read()? {
            Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.page = Page::About,
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

// impl Widget for &App {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let title = Line::from("GLAP".bold());
//         let block = Block::bordered()
//             .title(title.centered())
//             .border_set(border::DOUBLE);
//     }
// }
