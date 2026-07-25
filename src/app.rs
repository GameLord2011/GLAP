use std::io;

use ratatui::{DefaultTerminal, Frame};

#[derive(Default)]
enum Page {
    About,
    #[default] Player
}

#[derive(Default)]
pub struct App {
    page: Page,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| self.draw(frame))?;
        self.handle_events()?;
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        match self.page {
            Page::About => {
                
                frame.render_widget(widget, frame.area());
            },
            Page::Player => {}
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        todo!()
    }
}
