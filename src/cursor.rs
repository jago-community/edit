#[derive(Debug, PartialEq, Clone, Default)]
pub struct Cursor(usize, usize, usize);

impl Cursor {
    pub fn x(&self) -> usize {
        self.0
    }

    pub fn y(&self) -> usize {
        self.1
    }

    pub fn z(&self) -> usize {
        self.2
    }
}

impl From<(usize, usize, usize)> for Cursor {
    fn from((x, y, z): (usize, usize, usize)) -> Self {
        Cursor(x, y, z)
    }
}

use unicode_segmentation::UnicodeSegmentation;

impl Cursor {
    pub fn current<'a>(&self, buffer: &'a str) -> &'a str {
        let mut buffer = UnicodeSegmentation::graphemes(&buffer[self.z()..], true);

        match buffer.next() {
            Some(next) => next,
            None => "",
        }
    }
}

impl Cursor {
    pub fn forward_graphemes(&self, input: &str, count: usize) -> Self {
        let offset = self.z();

        let buffer = &input[offset..];

        let (mut dx, mut dy, mut dz) = (self.x(), 0, 0);

        let mut graphemes = UnicodeSegmentation::graphemes(buffer, true);

        for _ in 0..count {
            match graphemes.next() {
                Some("\n") => {
                    dz += "\n".len();
                    dy += 1;
                    dx = 0;
                }
                Some(grapheme) => {
                    dz += grapheme.len();
                    dx += grapheme.len();
                }
                None => {
                    break;
                }
            };
        }

        let mut next = Cursor::from((dx, self.y() + dy, self.z() + dz));

        if "\n" == next.current(&input) {
            match graphemes.next() {
                Some(grapheme) => {
                    next.2 += grapheme.len();
                    next.1 += 1;
                    next.0 = 0;
                }
                None => {}
            };
        }

        next
    }
}

#[test]
fn forward_graphemes() {
    let buffer = include_str!("../edit");

    macro_rules! assert_ {
        ($from:expr, $steps:expr,  $to:expr, $want:expr) => {
            let from: Cursor = $from.into();
            let to = from.forward_graphemes(buffer, $steps);
            let got = to.current(buffer);
            assert_eq!(
                to,
                $to.into(),
                "{:?} -> {} = got {:?} want {:?} {:?}",
                $from,
                $steps,
                to,
                $to,
                got
            );
            assert_eq!(
                got, $want,
                "{:?} -> {} = got {:?} want {:?}",
                $from, $steps, got, $want
            );
        };
    }

    let tests = vec![
        ((0, 0, 0), 1, (1, 0, 1), " "),
        ((1, 0, 1), 1, (2, 0, 2), "J"),
        ((2, 0, 2), 1, (3, 0, 3), "a"),
        ((3, 0, 3), 1, (4, 0, 4), "g"),
        ((4, 0, 4), 1, (5, 0, 5), "o"),
        ((5, 0, 5), 1, (0, 1, 7), "\n"),
        ((0, 0, 0), 1, (1, 0, 1), " "),
        ((1, 0, 1), 2, (3, 0, 3), "a"),
        ((3, 0, 3), 3, (0, 1, 7), "\n"),
        ((0, 1, 7), 4, (3, 2, 11), "C"),
        ((3, 2, 10), 5, (8, 2, 15), "e"),
    ];

    for (from, steps, to, want) in tests {
        assert_!(from, steps, to, want);
    }
}

use crate::unicode::split_line_bounds;

impl Cursor {
    pub fn forward_lines(&self, buffer: &str, count: usize) -> Self {
        let mut line_bounds = split_line_bounds(&buffer[self.z()..]).peekable();

        let (mut dx, mut dy, mut dz) = (0, self.y(), self.z());

        for _ in 0..count {
            if let Some(block) = line_bounds.next() {
                dz += block.len();
                dy += 1;
                dx = 0;

                match line_bounds.peek() {
                    Some(&"\n") => {
                        dz += "\n".len();
                        drop(line_bounds.next());
                    }
                    None => break,
                    _ => {}
                }
            } else {
                break;
            }
        }

        let next = Cursor::from((dx, dy, dz));

        let mut graphemes = UnicodeSegmentation::graphemes(&buffer[next.z()..], true).peekable();

        for _ in 0..self.x() {
            match graphemes.peek() {
                Some(&"\n") => {
                    break;
                }
                Some(grapheme) => {
                    dz += grapheme.len();
                    dx += grapheme.len();
                    drop(line_bounds.next());
                }
                None => break,
            };
        }

        (dx, dy, dz).into()
    }
}

#[test]
fn test_forward_lines() {
    let buffer = include_str!("../edit");

    macro_rules! assert_ {
        ($from:expr, $steps:expr,  $to:expr, $want:expr) => {
            let from: Cursor = $from.into();
            let to = from.forward_lines(buffer, $steps);
            let got = to.current(buffer);
            assert_eq!(
                to,
                $to.into(),
                "{:?} -> {} = got {:?} want {:?}",
                $from,
                $steps,
                to,
                $to
            );
            assert_eq!(
                got, $want,
                "{:?} -> {} = got {:?} want {:?}",
                $from, $steps, got, $want
            );
        };
    }

    let tests = vec![
        ((0, 0, 0), 1, (0, 1, 7), "\n"),
        ((0, 0, 0), 2, (0, 2, 8), ">"),
        ((3, 0, 0), 1, (0, 1, 7), "\n"),
        ((3, 0, 0), 2, (3, 2, 11), "C"),
    ];

    for (from, steps, to, want) in tests {
        assert_!(from, steps, to, want);
    }
}
