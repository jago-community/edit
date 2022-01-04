#[derive(Default)]
pub struct Document<'a> {
    buffer: &'a [u8],
    lense: Lense,
    position: usize,
    point: (usize, usize),
}

pub enum Lense {
    Graphemes,
    Words,
    Sentences,
    Lines,
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

        let tokens: Box<dyn Iterator<Item = &str>> = match self.lense {
            Lense::Graphemes => Box::new(UnicodeSegmentation::graphemes(buffer, true)),
            Lense::Words => Box::new(UnicodeSegmentation::split_word_bounds(buffer)),
            Lense::Sentences => Box::new(UnicodeSegmentation::split_sentence_bounds(buffer)),
            Lense::Lines => Box::new(buffer.lines()),
        };

        let mut color_picker = ColorPicker::new();

        tokens
            .map(|token| {
                SetForegroundColor(color_picker.pick()).write_ansi(out)?;
                Print(token).write_ansi(out)?;

                if token == "\n" {
                    MoveToColumn(0).write_ansi(out)?;
                }

                Ok(())
            })
            .find(|result: &Result<(), std::fmt::Error>| result.is_err())
            .unwrap_or(Ok(()))?;

        SetForegroundColor(color_picker.pick()).write_ansi(out)?;
        MoveToColumn(0).write_ansi(out)?;
        Print(format!(
            "\n{:?} {} {:?}",
            self.buffer[self.position] as char, self.position, self.point
        ))
        .write_ansi(out)?;

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
                self.step_backward(Lense::Graphemes).unwrap();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: _,
            }) => {
                self.step_forward(Lense::Lines).unwrap();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: _,
            }) => {
                self.step_forward(Lense::Lines).unwrap();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: _,
            }) => {
                self.step_forward(Lense::Graphemes).unwrap();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                self.lense = match self.lense {
                    Lense::Graphemes => Lense::Words,
                    Lense::Words => Lense::Sentences,
                    Lense::Sentences => Lense::Lines,
                    Lense::Lines => Lense::Graphemes,
                };
            }
            _ => {}
        }
    }

    pub fn focus(&mut self, point: (usize, usize)) {
        self.point = point;
    }

    fn step_forward(&mut self, lense: Lense) -> Result<(), Error> {
        let buffer = &self.buffer[self.position + 1..];

        if buffer.len() == 0 {
            return Ok(());
        }

        let buffer = unsafe { std::str::from_utf8_unchecked(buffer) };

        let mut tokens: Box<dyn Iterator<Item = &str>> = match &lense {
            Lense::Graphemes => Box::new(UnicodeSegmentation::graphemes(buffer, true)),
            Lense::Words => Box::new(UnicodeSegmentation::split_word_bounds(buffer)),
            Lense::Sentences => Box::new(UnicodeSegmentation::split_sentence_bounds(buffer)),
            Lense::Lines => Box::new(buffer.lines()),
        };

        match (&lense, tokens.next()) {
            (&Lense::Graphemes, Some("\n")) => {
                self.position += 1;
                self.point.0 = 0;
                self.point.1 += 1;
            }
            (&Lense::Graphemes, Some(grapheme)) => {
                self.position += grapheme.len();
                self.point.0 += grapheme.len();
            }
            (&Lense::Lines, Some(line)) => {
                self.position += line.len() + 1;
                self.point.0 = 0;
                self.point.1 += 1;
            }
            _ => {}
        };

        Ok(())
    }

    fn step_backward(&mut self, lense: Lense) -> Result<(), Error> {
        let buffer = &self.buffer[..self.position];

        if buffer.len() == 0 {
            return Ok(());
        }

        let buffer = unsafe { std::str::from_utf8_unchecked(buffer) };

        let mut tokens: Box<dyn Iterator<Item = &str>> = match &lense {
            Lense::Graphemes => Box::new(UnicodeSegmentation::graphemes(buffer, true)),
            Lense::Words => Box::new(UnicodeSegmentation::split_word_bounds(buffer)),
            Lense::Sentences => Box::new(UnicodeSegmentation::split_sentence_bounds(buffer)),
            Lense::Lines => Box::new(UnicodeSegmentation::split_sentence_bounds(buffer)),
        };

        match (&lense, tokens.next()) {
            (&Lense::Graphemes, Some("\n")) => {
                self.position += 1;
                self.point.0 = 0;
                self.point.1 += 1;
            }
            (&Lense::Graphemes, Some(grapheme)) => {
                self.position += grapheme.len();
                self.point.0 += grapheme.len();
            }
            (&Lense::Lines, Some(line)) => {
                self.position += line.len() + 1;
                self.point.0 = 0;
                self.point.1 += 1;
            }
            _ => {}
        };

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

struct ULineBounds<'a> {
    cursor: usize,
    source: &'a str,
}

#[test]
fn test_unicode_lines() {
    let buffer = "Hello, world!\nHow are you?\nI'm fine.\n";

    let mut lines = ULineBounds {
        cursor: 0,
        source: buffer,
    };

    assert_eq!(lines.next(), Some("Hello, world!"));
    assert_eq!(lines.next(), Some("\n"));
    assert_eq!(lines.next(), Some("How are you?"));
    assert_eq!(lines.next(), Some("\n"));
    assert_eq!(lines.next(), Some("I'm fine."));
    assert_eq!(lines.next(), Some("\n"));
    assert_eq!(lines.next(), None);
}

impl<'a> Iterator for ULineBounds<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let mut cursor = self.cursor;
        let offset = self.cursor;

        let mut word_bounds =
            UnicodeSegmentation::split_word_bound_indices(dbg!(&self.source[self.cursor..]));

        while let Some((index, next)) = word_bounds.next() {
            cursor = index + offset;

            if next == "\n" {
                break;
            }
        }

        let range = dbg!(self.cursor..cursor);

        self.cursor = cursor;

        self.source.get(range)
    }
}
