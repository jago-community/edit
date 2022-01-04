use crdts::{CmRDT, List};

pub struct Context {
    document: List<char, usize>,
    cursor: usize,
    point: (usize, usize),
}

impl Context {
    pub fn new(buffer: &[u8]) -> Self {
        Self {
            document: {
                let mut document = List::new();

                for byte in buffer {
                    document.apply(document.append(*byte, 0));
                }

                document
            },
            cursor: 0,
            point: (0, 0),
        }
    }
}

use unicode_segmentation::UnicodeSegmentation;

impl Context {
    pub fn step_forward(&mut self, count: usize) {
        UnicodeSegmentation::graphemes(self.document.read::<String>(), true);
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent};

impl Context {
    pub fn handle(&mut self, event: &Event) {
        match &event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                ..
            }) => {
                // self.step_backward_bytes(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => {
                // self.step_forward_lines(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) => {
                // self.step_backward_lines(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) => {
                // self.walk_forward(1);
            }
            _ => {}
        };
    }
}

use crossterm::{
    cursor::{MoveTo, MoveToColumn},
    terminal::{Clear, ClearType},
    Command,
};

use unicode_segmentation::UnicodeSegmentation;

impl Command for Context {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        Clear(ClearType::All).write_ansi(out)?;
        MoveTo(0, 0).write_ansi(out)?;

        let document = self.document.clone().read_into::<Vec<_>>();

        for (index, grapheme) in
            unsafe { std::str::from_utf8_unchecked(&document) }.grapheme_indices(true)
        {
            crossterm::style::SetForegroundColor(crossterm::style::Color::AnsiValue(ansi_color(
                index,
            )))
            .write_ansi(out)?;

            out.write_str(grapheme)?;

            if grapheme == "\n" {
                MoveToColumn(0).write_ansi(out)?;
            }
        }

        MoveTo(0, 0).write_ansi(out)?;

        Ok(())
    }
}

fn ansi_color(index: usize) -> u8 {
    (index % 230) as u8
}
