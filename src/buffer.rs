use std::collections::HashSet;

#[derive(Default)]
pub struct Buffer {
    bytes: Vec<u8>,
    new_lines: HashSet<usize>,
    cursor: Cursor,
}

impl Buffer {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
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

    fn move_backward_bytes(&mut self, count: usize) -> Option<usize> {
        let next = self.cursor.position.checked_sub(count)?;

        for position in next..self.cursor.position {
            if self.bytes.get(position) == Some(&b'\n') {
                self.new_lines.insert(position);
                self.cursor.y = self.cursor.y.checked_sub(1)?;

                let mut steps_from_previous = 0;

                for index in (1..position).rev() {
                    if self.bytes.get(index) == Some(&b'\n') {
                        break;
                    } else {
                        steps_from_previous += 1;
                    }
                }

                self.cursor.x = steps_from_previous;
            } else {
                self.cursor.x = self.cursor.x.checked_sub(1)?;
            }
        }

        let count = self.cursor.position - next;

        self.cursor.position = next;

        Some(count)
    }

    /*
    fn move_forward_lines(&mut self, count: usize) {
        let mut saw = 0;

        for index in self.cursor.position..self.bytes.len() {
            if self.bytes.get(index) == Some(&b'\n') {
                saw += 1;

                if saw == count {
                    break;
                }
            } else {
                self.cursor.x += 1;
            }

            self.cursor.position += 1;
        }

        self.cursor.y += saw;

        let mut dx = 0;

        for index in self.cursor.position..self.cursor.position + self.cursor.x {
            if index == self.cursor.x || self.bytes.get(index) == Some(&b'\n') {
                break;
            } else {
                dx += 1;
            }
        }

        self.cursor.x = dx;
    }
    */

    fn move_forward_lines(&mut self, count: usize) -> Option<usize> {
        let mut saw = 0;

        for _ in self.cursor.position..self.bytes.len() - 1 {
            if self.new_lines.contains(&self.cursor.position) {
                saw += 1;
            }

            self.move_forward_bytes(1)?;

            if saw == count {
                break;
            }
        }

        Some(count)
    }

    fn move_backward_lines(&mut self, count: usize) {
        unimplemented!();
    }
}

#[test]
fn test_move_some_bytes() {
    let input = "# Jago

> `Canker` but communist.

## Intro

The name Alec Thompson is one that most of us know for one reason or another. The same face might come to mind for each of us. 

## Canker

Canker was founded by one of the Alec Thompsons.
";

    let mut buffer = Buffer::new(input.as_bytes().into());

    for index in 0.."# Jago".len() {
        assert_eq!(index, buffer.cursor.position);
        assert_eq!(index, buffer.cursor.x);
        assert_eq!(0, buffer.cursor.y);
        buffer.move_forward_bytes(1);
    }

    assert_eq!(0, buffer.new_lines.len());
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
}
