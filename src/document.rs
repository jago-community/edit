#[derive(Default)]
pub struct Document<'a> {
    buffer: &'a [u8],
    lense: Lense,
    position: usize,
    point: (usize, usize),
}

pub enum Lense {
    Graphemes,
    Sentences,
}

impl Default for Lense {
    fn default() -> Self {
        Lense::Graphemes
    }
}

#[derive(Default, Clone)]
pub struct Cursor {
    position: usize,
    x: usize,
    y: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Machine {0}")]
    Machine(#[from] std::io::Error),
}

impl<'a> Document<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            lense: Lense::Sentences,
            position: 0,
            point: (0, 0),
        }
    }
}

use unicode_segmentation::UnicodeSegmentation;

use crossterm::{
    cursor::{MoveTo, MoveToColumn},
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
    Command,
};

impl<'a> Command for Document<'a> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        Clear(ClearType::All).write_ansi(out)?;
        MoveTo(0, 0).write_ansi(out)?;

        let buffer = unsafe { std::str::from_utf8_unchecked(&self.buffer) };

        let tokens: Box<dyn Iterator<Item = (usize, &str)>> = match self.lense {
            Lense::Graphemes => Box::new(UnicodeSegmentation::grapheme_indices(buffer, true)),
            Lense::Sentences => Box::new(UnicodeSegmentation::split_sentence_bound_indices(buffer)),
        };

        let mut color_picker = ColorPicker::new();

        tokens
            .map(|(_, token)| {
                SetForegroundColor(color_picker.pick()).write_ansi(out)?;
                Print(token).write_ansi(out)?;

                if token == "\n" {
                    MoveToColumn(0).write_ansi(out)?;
                }

                Ok(())
            })
            .find(|result: &Result<(), std::fmt::Error>| result.is_err())
            .unwrap_or(Ok(()))?;

        MoveTo(self.point.0 as u16, self.point.1 as u16).write_ansi(out)?;

        Ok(())
    }
}

use rand::rngs::ThreadRng;

struct ColorPicker {
    rng: ThreadRng,
    seq: [usize; 231],
}

impl ColorPicker {
    fn new() -> Self {
        let mut seq = [0; 231];

        for i in 0..seq.len() {
            seq[i] = i;
        }

        Self {
            rng: Default::default(),
            seq,
        }
    }
}

use rand::seq::SliceRandom;

impl ColorPicker {
    fn pick(&mut self) -> Color {
        Color::AnsiValue(*self.seq.choose(&mut self.rng).unwrap_or(&231) as u8)
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

impl<'a> Document<'a> {
    pub fn handle(&mut self, event: &Event) {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: _,
            }) => {
                self.step_backward().unwrap();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: _,
            }) => {
                self.step_forward().unwrap();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                self.lense = match self.lense {
                    Lense::Graphemes => Lense::Sentences,
                    Lense::Sentences => Lense::Graphemes,
                };
            }
            _ => {}
        }
    }

    pub fn focus(&mut self, point: (usize, usize)) {
        self.point = point;
    }

    fn step_forward(&mut self) -> Result<(), Error> {
        let buffer = &self.buffer[self.position..];

        if buffer.len() == 0 {
            return Ok(());
        }

        let buffer = unsafe { std::str::from_utf8_unchecked(buffer) };

        let mut graphemes = UnicodeSegmentation::graphemes(buffer, true);

        match graphemes.next() {
            Some("\n") => {
                self.position += 1;
                self.point.0 = 0;
                self.point.1 += 1;
            }
            Some(grapheme) => {
                self.position += grapheme.len();
                self.point.0 += grapheme.len();
            }
            None => {}
        };

        Ok(())
    }

    fn step_backward(&mut self) -> Result<(), Error> {
        let buffer = &self.buffer[..self.position];

        if buffer.len() == 0 {
            return Ok(());
        }

        let buffer = unsafe { std::str::from_utf8_unchecked(buffer) };

        let mut graphemes = UnicodeSegmentation::graphemes(buffer, true);

        match graphemes.next_back() {
            Some("\n") => {
                self.position -= 1;
                self.point.1 -= 1;

                let mut from_previous_line = 0;

                while let Some(previous_grapheme) = graphemes.next_back() {
                    if previous_grapheme == "\n" {
                        break;
                    }

                    from_previous_line += previous_grapheme.len();
                }

                self.point.0 = from_previous_line;
            }
            Some(grapheme) => {
                self.position -= grapheme.len();
                self.point.0 -= grapheme.len();
            }
            None => {}
        };

        Ok(())
    }
}
