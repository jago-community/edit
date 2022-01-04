#[derive(Default)]
pub struct Document<'a> {
    source: &'a str,
    buffer: String,
    focus: (u16, u16),
    cursor: usize,
    perspective: Perspective,
    events: Vec<Event>,
    trace: Vec<String>,
    errors: Vec<Error>,
}

enum Perspective {
    Before,
    After,
}

impl Default for Perspective {
    fn default() -> Self {
        Perspective::Before
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Machine {0}")]
    Machine(#[from] std::io::Error),
    #[error("String {0}")]
    String(#[from] std::string::FromUtf8Error),
    #[error("Incomplete")]
    Incomplete,
}

use crossterm::{
    cursor::{MoveTo, MoveToColumn},
    event::{Event, KeyCode, KeyEvent},
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
    Command,
};

use std::io::Write;

impl<'a> Document<'a> {
    pub fn new(source: &'a [u8]) -> Result<Self, Error> {
        Ok(Self {
            source: unsafe { std::str::from_utf8_unchecked(source) },
            buffer: String::from_utf8(source.into())?,
            ..Default::default()
        })
    }

    pub fn focus(&mut self, focus: (u16, u16)) {
        self.focus = focus;
    }

    // TODO: handle events (wraps to next line if past end of line)

    pub fn handle(&mut self, event: Event, mut _output: impl Write) -> Result<(), Error> {
        self.trace = vec![];

        match &event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => {
                self.set_position((self.focus.0, self.focus.1 + 1))?;
            }
            _ => {}
        };

        self.events.push(event);

        Ok(())
    }

    pub fn caught(&mut self, error: Error) {
        self.errors.push(error);
    }

    fn set_position(&mut self, (x, y): (u16, u16)) -> Result<(), Error> {
        while y != self.focus.1 {
            let next = if y > self.focus.1 {
                self.cursor.checked_add(1)
            } else {
                self.cursor.checked_sub(1)
            }
            .ok_or(Error::Incomplete)?;

            if Some(&b'\n') == self.source.as_bytes().get(next) {
                self.focus.1 = if y > self.focus.1 {
                    self.focus.1.checked_add(1).ok_or(Error::Incomplete)
                } else {
                    self.focus.1.checked_sub(1).ok_or(Error::Incomplete)
                }?;

                self.focus.0 = 0;
            }

            self.cursor = next;

            if self.cursor == self.source.len() - 1 {
                break;
            }
        }

        while x != self.focus.0 {
            println!("{} {:?} {}", x, self.focus, self.cursor);

            let mut next = if x > self.focus.0 {
                self.cursor.checked_add(1)
            } else {
                self.cursor.checked_sub(1)
            }
            .ok_or(Error::Incomplete)?;

            let mut up = false;

            if Some(&b'\n') != self.source.as_bytes().get(next) {
                self.focus.0 = if x > self.focus.0 {
                    self.focus.0.checked_add(1).ok_or(Error::Incomplete)
                } else {
                    up = true;
                    self.focus.0.checked_sub(1).ok_or(Error::Incomplete)
                }?;
            }

            if x == self.focus.0 && up {
                let mut rest = 0;

                while Some(&b'\n') != self.source.as_bytes().get(self.cursor - rest) {
                    rest += 1;
                }

                next = rest;
            }

            self.cursor = next;

            if self.cursor == self.source.len() - 1 {
                break;
            }
        }

        dbg!(self.focus);

        /*
        while y > self.focus.1 + dy {
            let d = self.cursor.checked_add(1).ok_or(Error::Incomplete)?;

            if self.source.as_bytes().get(d) == Some(&b'\n') {
                dy = dy.checked_add(1).ok_or(Error::Incomplete)?;
            }

            if d < self.source.len() {
                self.cursor = d;
            } else {
                self.cursor = self.source.len() - 1;
                break;
            }
        }

        self.focus.1 = self.focus.1.checked_add(dy).ok_or(Error::Incomplete)?;
        */

        Ok(())
    }
}

#[test]
#[ignore]
fn test_set_position() {
    dbg!(b"# Jago

> `Canker` but communist.

## Intro

The name Alec Thompson is one that most of us know for one reason or another. The same face might come to mind for each of us. 

## Canker

Canker was founded by one of the Alec Thompsons.
".len());
    let mut doc = Document::new(b"# Jago

> `Canker` but communist.

## Intro

The name Alec Thompson is one that most of us know for one reason or another. The same face might come to mind for each of us. 

## Canker

Canker was founded by one of the Alec Thompsons.
").unwrap();

    let tests = vec![
        ((0, 0), 0, (0, 0)),
        (
            (0, 1),
            "# Jago
"
            .len()
                - 1,
            (0, 1),
        ),
        (
            (0, 2),
            "# Jago

"
            .len()
                - 1,
            (0, 2),
        ),
        (
            (10, 2),
            "# Jago

> `Canker`"
            .len()
                - 1,
            (10, 2),
        ),
        (
            (0, 3),
            "# Jago

> `Canker` but communist.
"
            .len()
                - 1,
                (0, 3)
        ),
        (
            (0, 13),
            "# Jago

> `Canker` but communist.

## Intro

The name Alec Thompson is one that most of us know for one reason or another. The same face might come to mind for each of us. 

## Canker

Canker was founded by one of the Alec Thompsons.
"
            .len()
                - 1,
                (0, 11),
        ),
        (
            (15, 2),
            "# Jago

> `Canker` but c"
            .len()
                ,
                (15, 2),
        ),
    ];

    for (point, stretch, to) in tests {
        doc.set_position(point).unwrap();

        println!("{} {:?}", doc.cursor, doc.focus);
        assert_eq!(doc.cursor, stretch);
        assert_eq!(doc.focus, to);
    }
}

impl<'a> Document<'a> {
    fn color(&self, index: usize) -> u8 {
        (index % 230) as u8
    }
}

use unicode_segmentation::UnicodeSegmentation;

impl<'a> Command for Document<'a> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        Clear(ClearType::All).write_ansi(out)?;
        MoveTo(0, 0).write_ansi(out)?;

        let (mut x, mut y) = (0, 0);

        for (index, grapheme) in self.source.grapheme_indices(true) {
            let focus = self.focus == (x, y);

            crossterm::style::SetForegroundColor(crossterm::style::Color::AnsiValue(
                self.color(index),
            ))
            .write_ansi(out)?;

            out.write_str(if focus && grapheme == " " {
                "_"
            } else {
                grapheme
            })?;

            if grapheme == "\n" {
                x = 0;
                y += 1;
                MoveToColumn(0).write_ansi(out)?;
            } else {
                x += grapheme.len() as u16;
            }
        }

        MoveTo(0, y + 1).write_ansi(out)?;
        SetForegroundColor(Color::Green).write_ansi(out)?;
        Print(format!(
            "{} ({}, {})\n\n",
            self.cursor, self.focus.0, self.focus.1
        ))
        .write_ansi(out)?;
        SetForegroundColor(Color::Blue).write_ansi(out)?;
        for item in &self.events {
            out.write_fmt(format_args!("{:?}\n{}", item, MoveToColumn(0)))?;
        }
        SetForegroundColor(Color::Red).write_ansi(out)?;
        for item in &self.errors {
            out.write_fmt(format_args!("{:?}\n{}", item, MoveToColumn(0)))?;
        }
        SetForegroundColor(Color::Magenta).write_ansi(out)?;
        for item in &self.trace {
            out.write_fmt(format_args!("{:?}\n{}", item, MoveToColumn(0)))?;
        }
        MoveTo(self.focus.0, self.focus.1).write_ansi(out)?;

        Ok(())
    }
}
