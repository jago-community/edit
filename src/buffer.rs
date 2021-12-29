use std::collections::HashSet;

#[derive(Default)]
pub struct Buffer {
    bytes: Vec<u8>,
    new_lines: HashSet<usize>,
    cursor: Cursor,
}

impl Buffer {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.into(),
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct Cursor {
    position: usize,
    y: usize,
    x: usize,
}

impl Buffer {
    fn move_forward_bytes(&mut self, count: usize) -> Option<usize> {
        let next = self.cursor.position.checked_add(count)?;

        if next >= self.bytes.len() {
            return None;
        }

        for position in self.cursor.position + 1..=next {
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

impl Buffer {
    fn move_backward_bytes(&mut self, count: usize) -> Option<usize> {
        let mut saw = 0;

        while let Some(position) = self.cursor.position.checked_sub(count) {
            dbg!(self.bytes.get(position).map(|c| *c as char));
            if self.bytes.get(position) == Some(&b'\n') {
                self.new_lines.insert(position);
                self.cursor.y -= 1;

                let mut steps_from_previous_line = 0;

                for index in (1..position).rev() {
                    if self.bytes.get(index) == Some(&b'\n') {
                        break;
                    } else {
                        steps_from_previous_line += 1;
                    }
                }

                dbg!(steps_from_previous_line);

                self.cursor.x = steps_from_previous_line;
            } else {
                dbg!(&self.cursor);
                self.cursor.x -= 1;
            }

            saw += 1;

            self.cursor.position = position;

            if saw == count {
                break;
            }
        }

        Some(count)

        /*
        let next = self.cursor.position.checked_sub(count)?;

        for position in (next..=self.cursor.position).rev() {
            if self.bytes.get(position) == Some(&b'\n') {
                self.new_lines.insert(position);
                self.cursor.y = self.cursor.y.checked_sub(1)?;

                let mut steps_from_previous_line = 0;

                for index in (1..position).rev() {
                    if self.bytes.get(index) == Some(&b'\n') {
                        break;
                    } else {
                        steps_from_previous_line += 1;
                    }
                }

                dbg!(steps_from_previous_line);

                self.cursor.x = steps_from_previous_line;
            } else {
                self.cursor.x = self.cursor.x.checked_sub(1)?;
            }

            self.cursor.position = position;
        }

        let count = self.cursor.position - next;
        */

        //self.cursor.position = next;

        //Some(count)
    }
}

impl Buffer {
    fn move_forward_lines(&mut self, count: usize) -> Option<usize> {
        let mut saw = 0;

        let dx = self.cursor.x;

        for _ in self.cursor.position..self.bytes.len() - 1 {
            if self.new_lines.contains(&self.cursor.position) {
                saw += 1;
            }

            self.move_forward_bytes(1)?;

            if saw == count {
                break;
            }
        }

        //for _ in 0..dx {
        //if self.new_lines.contains(&self.cursor.position) {
        //break;
        //}
        //self.move_forward_bytes(1)?;
        //}

        Some(count)
    }

    fn move_backward_lines(&mut self, count: usize) -> Option<usize> {
        let mut saw = 0;

        for _ in 0..self.cursor.position {
            self.move_backward_bytes(1)?;

            if self.new_lines.contains(&self.cursor.position) {
                saw += 1;
            }

            if saw == count {
                break;
            }
        }

        //self.move_backward_bytes(dbg!(&self.cursor).x)?;

        //let mut line_length = 0;

        //for _ in 0..self.cursor.position {
        //self.move_backward_bytes(1)?;
        //line_length += 1;

        //if self.new_lines.contains(&self.cursor.position) {
        //break;
        //}
        //}

        //dbg!((want_x, line_length));

        //let diff = std::cmp::min(want_x, line_length);

        //self.move_forward_bytes(diff)?;

        Some(count)
    }
}

#[test]
fn test_move_some_bytes() {
    let input = b"# Jago

> `Canker` but communist.

## Intro

The name Alec Thompson is one that most of us know for one reason or another. The same face might come to mind for each of us. 

## Canker

Canker was founded by one of the Alec Thompsons.
";

    let mut buffer = Buffer::new(input);

    let b = b"# Jago";

    assert_eq!(0, buffer.cursor.position);
    assert_eq!(0, buffer.cursor.x);
    assert_eq!(0, buffer.cursor.y);

    for index in 1..="# Jago".len() {
        buffer.move_forward_bytes(1);
        assert_eq!(index, buffer.cursor.position);
        assert_eq!(index, buffer.cursor.x);
        assert_eq!(0, buffer.cursor.y);
    }

    buffer.move_forward_bytes(1);
    assert_eq!(
        "# Jago
"
        .len()
            - 1,
        buffer.cursor.position
    );
    assert_eq!(0, buffer.cursor.x);
    assert_eq!(1, buffer.cursor.y);

    buffer.move_backward_bytes(1);
    assert_eq!("# Jago".len(), buffer.cursor.position);
    assert_eq!("# Jago".len(), buffer.cursor.x);
    assert_eq!(0, buffer.cursor.y);
    dbg!(&buffer.cursor);
    buffer.move_backward_bytes(1);
    assert_eq!("# Jag".len() - 1, buffer.cursor.position);
    assert_eq!("# Jag".len() - 1, buffer.cursor.x);
    assert_eq!(0, buffer.cursor.y);

    /*
        buffer.move_forward_bytes(2);
        assert_eq!(2, buffer.new_lines.len());
        assert_eq!(0, buffer.cursor.x);
        assert_eq!(2, buffer.cursor.y);
        assert_eq!("# Jago".len() + 2, buffer.cursor.position);

        buffer.move_backward_bytes(1);
        assert_eq!(2, buffer.new_lines.len());
        assert_eq!(0, buffer.cursor.x);
        assert_eq!(1, buffer.cursor.y);
        assert_eq!("# Jago".len() + 1, buffer.cursor.position);

        buffer.move_backward_bytes(1);
        assert_eq!(2, buffer.new_lines.len());
        assert_eq!("# Jago".len() - 1, buffer.cursor.x);
        assert_eq!(0, buffer.cursor.y);
        assert_eq!("# Jago".len(), buffer.cursor.position);

        buffer.move_forward_bytes(
            "

    > `Canker` but communist.
    "
            .len(),
        );
        assert_eq!(
            "# Jago

    > `Canker` but communist.
    "
            .len(),
            buffer.cursor.position
        );
        assert_eq!(3, buffer.cursor.y);
        assert_eq!(0, buffer.cursor.x);
        buffer.move_backward_bytes(1);
        assert_eq!(
            "# Jago

    > `Canker` but communist."
                .len(),
            buffer.cursor.position
        );
        assert_eq!(2, buffer.cursor.y);
        assert_eq!("> `Canker` but communist.".len(), buffer.cursor.x);
        buffer.move_forward_lines(1);
        assert_eq!(
            "# Jago

    > `Canker` but communist.
    "
            .len(),
            buffer.cursor.position
        );
        assert_eq!(3, buffer.cursor.y);
        assert_eq!(0, buffer.cursor.x);
        dbg!(&buffer.cursor);
        buffer.move_forward_lines(1);
        dbg!(&buffer.cursor);
        assert_eq!(
            "# Jago

    > `Canker` but communist.

    "
            .len(),
            buffer.cursor.position
        );
        assert_eq!(4, buffer.cursor.y);
        assert_eq!(0, buffer.cursor.x);*/
    /*
        buffer.move_forward_lines(1);
        assert_eq!(
            "# Jago
    "
            .len(),
            buffer.cursor.position
        );
        assert_eq!(1, buffer.cursor.y);
        assert_eq!(0, buffer.cursor.x);
        buffer.move_forward_lines(1);
        assert_eq!(
            "# Jago

    "
            .len(),
            buffer.cursor.position
        );
        assert_eq!(2, buffer.cursor.y);
        assert_eq!(0, buffer.cursor.x);
        buffer.move_backward_lines(1);
        assert_eq!(
            "# Jago
    "
            .len(),
            buffer.cursor.position
        );
        assert_eq!(1, buffer.cursor.y);
        assert_eq!(0, buffer.cursor.x);
        buffer.move_backward_lines(1);
        assert_eq!(0, buffer.cursor.position);
        assert_eq!(0, buffer.cursor.x);
        assert_eq!(0, buffer.cursor.y);
        buffer.move_forward_lines(2);
        assert_eq!(
            "# Jago

    > `Ca"
                .len(),
            buffer.cursor.position
        );
        assert_eq!(2, buffer.cursor.y);
        assert_eq!(5, buffer.cursor.x);
        buffer.move_backward_bytes(2);
        assert_eq!(
            "# Jago

    > `"
            .len(),
            buffer.cursor.position
        );
        assert_eq!("> `".len(), buffer.cursor.x);
        buffer.move_backward_lines(2);
        assert_eq!("# Ja".len(), buffer.cursor.position);
        assert_eq!("# Ja".len() - 1, buffer.cursor.x);
        assert_eq!(0, buffer.cursor.y);
        */
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
                code: KeyCode::Char('l'),
                ..
            }) => {
                self.move_forward_bytes(1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                ..
            }) => {
                self.move_backward_bytes(1);
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
            } else {
                x += grapheme.len();
            }
        }

        MoveTo(0, (y + 1) as u16).write_ansi(out)?;
        SetForegroundColor(Color::Green).write_ansi(out)?;
        Print(format!(
            "{:?} {} ({}, {})\n\n",
            self.bytes[self.cursor.position] as char,
            self.cursor.position,
            self.cursor.x,
            self.cursor.y
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
