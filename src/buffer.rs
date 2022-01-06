use crate::cursor::Cursor;

pub struct Buffer {
    source: String,
    cursor: Cursor,
    mode: Mode,
    scale: Option<u32>,
}

enum Mode {
    Graphemes,
    Lines,
}

use unicode_segmentation::UnicodeSegmentation;

impl Buffer {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            source: input.into(),
            cursor: Cursor::default(),
            mode: Mode::Graphemes,
            scale: None,
        }
    }

    pub fn current(&self) -> &str {
        UnicodeSegmentation::graphemes(&self.source[..], true)
            .next()
            .unwrap_or("")
    }

    fn scale(&self) -> u32 {
        self.scale.unwrap_or(1)
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

impl Buffer {
    pub fn handle(&mut self, event: &Event) {
        let mut next_scale = None;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: _,
            }) => {
                //self.step_backward(Lense::Graphemes).unwrap();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: _,
            }) => {
                self.cursor = self
                    .cursor
                    .forward_lines(&self.source, self.scale() as usize);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: _,
            }) => {
                //self.step_backward(Lense::Lines).unwrap();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: _,
            }) => {
                self.cursor = self
                    .cursor
                    .forward_graphemes(&self.source, self.scale() as usize);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                self.mode = match self.mode {
                    Mode::Graphemes => Mode::Lines,
                    Mode::Lines => Mode::Graphemes,
                };
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(code),
                modifiers: _,
            }) => {
                if let Some(digit) = code.to_digit(10) {
                    if let Some(previous_scale) = self.scale {
                        next_scale = Some(previous_scale * 10 + digit);
                    } else {
                        next_scale = Some(digit);
                    }
                }
            }
            _ => {}
        };

        self.scale = next_scale;
    }
}

use crossterm::{
    cursor::{MoveTo, MoveToColumn},
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
    Command,
};
use itertools::Itertools;

impl Command for Buffer {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        Clear(ClearType::All).write_ansi(out)?;
        MoveTo(0, 0).write_ansi(out)?;

        let tokens: Box<dyn Iterator<Item = &str>> = match self.mode {
            Mode::Graphemes => Box::new(UnicodeSegmentation::graphemes(&self.source[..], true)),
            Mode::Lines => Box::new(
                UnicodeSegmentation::split_word_bound_indices(&self.source[..]).batching(|rest| {
                    let (mut stop, _) = rest.next()?;
                    let start = stop;

                    while let Some((next_stop, next_word)) = rest.next() {
                        stop = next_stop;

                        if next_word == "\n" {
                            stop += next_word.len();
                            break;
                        }
                    }

                    Some(&self.source[start..stop])
                }),
            ),
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
            "\n{:?} {:?} {}",
            self.current(),
            (self.cursor.x(), self.cursor.y()),
            self.cursor.z(),
        ))
        .write_ansi(out)?;

        MoveTo(self.cursor.x() as u16, self.cursor.y() as u16).write_ansi(out)?;

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
