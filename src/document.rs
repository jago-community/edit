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

    pub fn current(&self, lense: Lense) -> Option<&'a str> {
        let buffer = unsafe { std::str::from_utf8_unchecked(&self.buffer[self.position..]) };

        let mut tokens: Box<dyn Iterator<Item = &str>> = match lense {
            Lense::Graphemes => Box::new(UnicodeSegmentation::graphemes(buffer, true)),
            Lense::Words => Box::new(UnicodeSegmentation::split_word_bounds(buffer)),
            Lense::Sentences => Box::new(UnicodeSegmentation::split_sentence_bounds(buffer)),
            //Lense::Lines => Box::new(buffer.lines()),
            Lense::Lines => Box::new(ULineBounds::new(buffer)),
        };

        tokens.next()
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
            //Lense::Lines => Box::new(buffer.lines()),
            Lense::Lines => Box::new(ULineBounds::new(buffer)),
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
            "{} {:?}\n{:?}",
            self.position,
            self.point,
            self.current(Lense::Graphemes)
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
                self.step_backward(Lense::Lines).unwrap();
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
            Lense::Graphemes => Box::new(UnicodeSegmentation::graphemes(buffer, true).rev()),
            Lense::Words => Box::new(UnicodeSegmentation::split_word_bounds(buffer).rev()),
            Lense::Sentences => Box::new(UnicodeSegmentation::split_sentence_bounds(buffer)),
            Lense::Lines => Box::new(ULineBounds::new(buffer).rev()),
        };

        match (&lense, tokens.next()) {
            (&Lense::Graphemes, Some("\n")) => {
                self.position -= 1;
                self.point.0 = 0;
                self.point.1 -= 1;
            }
            (&Lense::Graphemes, Some(grapheme)) => {
                self.position -= grapheme.len();
                self.point.0 -= grapheme.len();
            }
            (&Lense::Lines, Some(line)) if self.point.1 > 0 => {
                self.position -= line.len();
                self.point.0 = 0;
                self.point.1 -= 1;
            }
            _ => {}
        };

        Ok(())
    }
}

#[test]
fn test_stepping() {
    let bytes = include_bytes!("../edit");

    let mut document = Document::new(bytes);

    document.step_forward(Lense::Graphemes).unwrap();

    assert_eq!(document.position, 1);
    assert_eq!(document.point, (1, 0));
    assert_eq!(document.current(Lense::Graphemes), Some(" "));

    document.step_backward(Lense::Graphemes).unwrap();

    assert_eq!(document.position, 0);
    assert_eq!(document.point, (0, 0));
    assert_eq!(document.current(Lense::Graphemes), Some("#"));

    document.step_backward(Lense::Graphemes).unwrap();

    assert_eq!(document.position, 0);
    assert_eq!(document.point, (0, 0));
    assert_eq!(document.current(Lense::Graphemes), Some("#"));

    document.step_forward(Lense::Lines).unwrap();

    assert_eq!(document.position, 6);
    assert_eq!(document.point, (0, 1));
    assert_eq!(document.current(Lense::Graphemes), Some("\n"));

    document.step_backward(Lense::Lines).unwrap();

    assert_eq!(document.position, 0);
    assert_eq!(document.point, (0, 0));
    assert_eq!(document.current(Lense::Graphemes), Some("#"));

    document.step_backward(Lense::Lines).unwrap();

    assert_eq!(document.position, 0);
    assert_eq!(document.point, (0, 0));
    assert_eq!(document.current(Lense::Graphemes), Some("#"));

    document.step_forward(Lense::Lines).unwrap();
    document.step_forward(Lense::Lines).unwrap();

    assert_eq!(document.position, 8);
    assert_eq!(document.point, (0, 3));
    assert_eq!(document.current(Lense::Graphemes), Some(">"));

    document.step_forward(Lense::Graphemes).unwrap();

    assert_eq!(document.position, 9);
    assert_eq!(document.point, (1, 2));
    assert_eq!(document.current(Lense::Graphemes), Some(" "));
}

use std::ops::Range;

struct ULineBounds<'a> {
    span: Range<usize>,
    source: &'a str,
}

impl<'a> ULineBounds<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            span: 0..source.len(),
            source,
        }
    }
}

#[test]
fn test_unicode_lines() {
    let buffer = "Hello, world!\nHow are you?\nI'm fine.\n";

    let mut lines = ULineBounds {
        span: 0..buffer.len(),
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
        let buffer = &self.source[self.span.clone()];

        let mut word_bounds = UnicodeSegmentation::split_word_bound_indices(buffer);

        let index = word_bounds
            .find(|(_, next)| *next == "\n")
            .map(|(index, _)| index)
            .or(Some(self.source.len()))?;

        let next_new_line = self.span.start + index;

        let range = self.span.start..next_new_line;

        let range = if range.is_empty() {
            range.start..range.end + 1
        } else {
            range
        };

        self.span.start = range.end;

        self.source.get(range)
    }
}

#[test]
fn test_unicode_lines_backward() {
    let buffer = "Hello, world!\nHow are you?\nI'm fine.\n";

    let mut lines = ULineBounds {
        span: 0..buffer.len(),
        source: buffer,
    };

    assert_eq!(lines.next_back(), Some("\n"));
    assert_eq!(lines.next_back(), Some("I'm fine."));
    assert_eq!(lines.next_back(), Some("\n"));
    assert_eq!(lines.next_back(), Some("How are you?"));
    assert_eq!(lines.next_back(), Some("\n"));
    assert_eq!(lines.next_back(), Some("Hello, world!"));
    assert_eq!(lines.next_back(), None);
}

impl<'a> DoubleEndedIterator for ULineBounds<'a> {
    fn next_back(&mut self) -> Option<&'a str> {
        let buffer = &self.source[self.span.clone()];

        let mut word_bounds = UnicodeSegmentation::split_word_bound_indices(buffer);

        let index = word_bounds
            .rfind(|(_, next)| *next == "\n")
            .map(|(index, _)| index + 1)
            .or(Some(0))?;

        let range = index..self.span.end;

        if range.start == 0 && range.end == 0 {
            return None;
        }

        let range = if range.is_empty() {
            range.start - 1..range.end
        } else {
            range
        };

        self.span.end = range.start;

        self.source.get(dbg!(range))
    }
}
