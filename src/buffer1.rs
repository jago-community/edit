#[cfg(test)]
static TEST_DOCUMENT: &'static [u8] = include_bytes!("../edit");

use std::collections::HashSet;

#[derive(Default)]
pub struct Buffer {
    bytes: Vec<u8>,
    new_lines: HashSet<usize>,
    cursor: Cursor,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Cursor {
    position: usize,
    y: usize,
    x: usize,
}

impl Buffer {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.into(),
            ..Default::default()
        }
    }

    pub fn get_char(&self, at: usize) -> Option<char> {
        self.bytes.get(at).map(|byte| *byte as char)
    }

    pub fn current_char(&self) -> Option<char> {
        self.get_char(self.cursor.position)
    }

    pub fn check_current_byte(&self, want: u8) -> bool {
        self.bytes
            .get(self.cursor.position)
            .map(|byte| *byte == want)
            .unwrap_or(false)
    }
}

#[test]
#[ignore]
fn test_step_forward_bytes() {
    let points = vec![
        Cursor {
            position: "# Ja".len(),
            x: 4,
            y: 0,
        },
        Cursor {
            position: "# Jag".len(),
            x: 5,
            y: 0,
        },
        Cursor {
            position: "# Jago".len(),
            x: 6,
            y: 0,
        },
        Cursor {
            position: "# Jago
"
            .len(),
            x: 0,
            y: 1,
        },
        Cursor {
            position: "# Jago

"
            .len(),
            x: 0,
            y: 2,
        },
        Cursor {
            position: "# Jago

>"
            .len(),
            x: 1,
            y: 2,
        },
    ];

    let mut buffer = Buffer {
        bytes: TEST_DOCUMENT.into(),
        cursor: points.get(0).unwrap().clone(),
        ..Default::default()
    };

    for point in &points[1..] {
        buffer.step_forward_bytes(1);
        assert_eq!(&buffer.cursor, point);
    }
}

impl Buffer {
    fn step_forward_bytes(&mut self, count: usize) -> Option<usize> {
        let next = self.cursor.position.checked_add(count)?;

        if next >= self.bytes.len() {
            return None;
        }

        for position in self.cursor.position..next {
            if self.bytes.get(position) == Some(&b'\n') {
                self.new_lines.insert(position);
                self.cursor.y += 1;
                self.cursor.x = 0;
            } else {
                self.cursor.x += 1;
            }
        }

        let count = next - self.cursor.position;

        self.cursor.position = next;

        Some(count)
    }
}

#[test]
#[ignore]
fn test_walk_forward() {
    let points = vec![
        Cursor {
            position: "# Ja".len(),
            x: 4,
            y: 0,
        },
        Cursor {
            position: "# Jag".len(),
            x: 5,
            y: 0,
        },
        Cursor {
            position: "# Jago
"
            .len(),
            x: 0,
            y: 1,
        },
        Cursor {
            position: "# Jago

"
            .len(),
            x: 0,
            y: 2,
        },
        Cursor {
            position: "# Jago

>"
            .len(),
            x: 1,
            y: 2,
        },
        Cursor {
            position: "# Jago

> "
            .len(),
            x: 2,
            y: 2,
        },
    ];

    let mut buffer = Buffer {
        bytes: TEST_DOCUMENT.into(),
        cursor: points.get(0).unwrap().clone(),
        ..Default::default()
    };

    for point in &points[1..] {
        buffer.walk_forward(1);
        assert_eq!(&buffer.cursor, point);
    }
}

impl Buffer {
    pub fn walk_forward(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step_forward_bytes(1).unwrap();
        }

        if self.current_char() == Some('\n') {
            self.step_forward_bytes(1).unwrap();
        }
    }
}

#[test]
#[ignore]
fn test_step_backward_bytes() {
    let points = vec![
        Cursor {
            position: "# Jago

> `Canker` but communist.

"
            .len(),
            x: 0,
            y: 4,
        },
        Cursor {
            position: "# Jago

> `Canker` but communist.
"
            .len(),
            x: 0,
            y: 3,
        },
        Cursor {
            position: "# Jago

> `Canker` but communist."
                .len(),
            x: "> `Canker` but communist.".len(),
            y: 2,
        },
    ];

    let mut buffer = Buffer {
        bytes: TEST_DOCUMENT.into(),
        cursor: points.get(0).unwrap().clone(),
        ..Default::default()
    };

    for point in &points[1..] {
        buffer.step_backward_bytes(1);
        println!("got {:?} = want {:?}", buffer.cursor, point);
        assert_eq!(&buffer.cursor, point);
    }

    let points = vec![
        Cursor {
            position: 10,
            x: 2,
            y: 2,
        },
        Cursor {
            position: 9,
            x: 1,
            y: 2,
        },
        Cursor {
            position: 8,
            x: 0,
            y: 2,
        },
        Cursor {
            position: 7,
            x: 0,
            y: 1,
        },
        Cursor {
            position: 6,
            x: 6,
            y: 0,
        },
        Cursor {
            position: 5,
            x: 5,
            y: 0,
        },
    ];

    let mut buffer = Buffer {
        bytes: TEST_DOCUMENT.into(),
        cursor: points.get(0).unwrap().clone(),
        ..Default::default()
    };

    for point in &points[1..] {
        buffer.step_backward_bytes(1);
        println!("{:?} = {:?}", buffer.cursor, point);
        assert_eq!(&buffer.cursor, point);
    }
}

impl Buffer {
    fn step_backward_bytes(&mut self, count: usize) -> Option<usize> {
        let mut saw = 0;

        for _ in 0..self.cursor.position {
            let next = self.cursor.position.checked_sub(1)?;

            saw += 1;

            if self.bytes.get(next) == Some(&b'\n') {
                self.new_lines.insert(next);
                self.cursor.y = self.cursor.y.checked_sub(1)?;

                let mut nth_from_last_line = 0;

                for _ in 0..=next {
                    nth_from_last_line += 1;

                    let previous = match next.checked_sub(1) {
                        Some(c) => c,
                        None => break,
                    };

                    if self.bytes.get(previous) == Some(&b'\n') {
                        break;
                    }
                }

                self.cursor.x = nth_from_last_line - 1;
            } else {
                self.cursor.x = self.cursor.x.checked_sub(1)?;
            }

            self.cursor.position = next;

            if saw == count {
                break;
            }
        }

        Some(self.cursor.position)
    }
}

#[test]
#[ignore]
fn test_step_forward_lines() {
    let points = vec![
        Cursor {
            position: "# J".len(),
            x: 2,
            y: 0,
        },
        Cursor {
            position: "# Jago
"
            .len()
                - 1,
            x: 6,
            y: 0,
        },
        Cursor {
            position: "# Jago

"
            .len(),
            x: 0,
            y: 2,
        },
    ];

    let mut buffer = Buffer {
        bytes: TEST_DOCUMENT.into(),
        cursor: points.get(0).unwrap().clone(),
        ..Default::default()
    };

    for point in &points[1..] {
        buffer.step_forward_lines(1);
        println!("got {:?} = want {:?}", buffer.cursor, point);
        assert_eq!(&buffer.cursor, point);
    }
}

impl Buffer {
    fn step_forward_lines(&mut self, count: usize) -> Option<usize> {
        let mut saw = 0;

        for _ in self.cursor.position..self.bytes.len() {
            self.step_forward_bytes(1)?;

            if self.check_current_byte(b'\n') {
                saw += 1;
            }

            if saw == count {
                break;
            }
        }

        Some(count)
    }
}

impl Buffer {
    fn step_backward_lines(&mut self, count: usize) -> Option<usize> {
        Some(count)
    }
}

use crossterm::{
    cursor::{MoveTo, MoveToColumn},
    event::{Event, KeyCode, KeyEvent},
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
    Command,
};

impl Buffer {
    pub fn handle(&mut self, event: &Event) {
        match &event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                ..
            }) => {
                self.step_backward_bytes(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => {
                self.step_forward_lines(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) => {
                self.step_backward_lines(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) => {
                self.walk_forward(1);
            }
            _ => {}
        };
    }
}

use unicode_segmentation::UnicodeSegmentation;

impl Command for Buffer {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        Clear(ClearType::All).write_ansi(out)?;
        MoveTo(0, 0).write_ansi(out)?;

        let (mut x, mut y) = (0, 0);
        let (mut dx, mut dy) = (self.cursor.x, self.cursor.y);

        for (index, grapheme) in
            unsafe { std::str::from_utf8_unchecked(&self.bytes) }.grapheme_indices(true)
        {
            let focus = (self.cursor.x, self.cursor.y) == (x, y);

            crossterm::style::SetForegroundColor(crossterm::style::Color::AnsiValue(ansi_color(
                index,
            )))
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

                if focus {
                    dx = 0;
                    dy += 1;
                }
            } else {
                x += grapheme.len();
            }
        }

        MoveTo(0, (y + 1) as u16).write_ansi(out)?;
        SetForegroundColor(Color::Green).write_ansi(out)?;
        Print(format!(
            "{:?} {} ({}, {}) -> ({}, {})\n\n",
            self.bytes[self.cursor.position] as char,
            self.cursor.position,
            self.cursor.x,
            self.cursor.y,
            dx,
            dy,
        ))
        .write_ansi(out)?;
        SetForegroundColor(Color::Blue).write_ansi(out)?;
        MoveTo(self.cursor.x as u16, self.cursor.y as u16).write_ansi(out)?;

        Ok(())
    }
}

fn ansi_color(index: usize) -> u8 {
    (index % 230) as u8
}
