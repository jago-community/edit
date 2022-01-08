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
        let mut next = input
            .get(self.z()..)
            .iter()
            .flat_map(|slice| slice.graphemes(true))
            .take(count)
            .fold(self.clone(), |cursor, grapheme| {
                let mut next = cursor;

                next.2 += grapheme.len();
                next.0 += grapheme.len();

                if "\n" == grapheme {
                    next.1 += 1;
                    next.0 = 0;
                }

                next
            });

        if next.current(input) == "\n" {
            next.2 += 1;
            next.1 += 1;
            next.0 = 0;
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

use itertools::{
    FoldWhile::{Continue, Done},
    Itertools,
};

impl Cursor {
    /*pub fn backward_graphemes(&self, input: &str, count: usize) -> Self {
        let offset = self.z();

        let graphemes = UnicodeSegmentation::graphemes(&input[..=offset], true);

        let (mut dx, dy, dz) = graphemes
            .clone()
            .rev()
            .fold_while((0, 0, 0), |(mut dx, mut dy, mut dz), grapheme| {
                match grapheme {
                    "\n" => {
                        dy += 1;
                        dz += grapheme.len();
                    }
                    _ => {
                        dx += grapheme.len();
                        dz += grapheme.len();
                    }
                };

                if dz >= count {
                    Done((dx, dy, dz))
                } else {
                    Continue((dx, dy, dz))
                }
            })
            .into_inner();

        let graphemes = UnicodeSegmentation::graphemes(&input[..offset - dz], true);

        if dz > dx {
            let to_previous_line = graphemes
                .rev()
                .fold_while(0, |to_previous_line, grapheme| {
                    if grapheme == "\n" {
                        Done(to_previous_line)
                    } else {
                        Continue(to_previous_line + grapheme.len())
                    }
                })
                .into_inner();

            dx = to_previous_line;
        } else {
            dx = self.x() - dx;
        }

        let mut next = Cursor::from((dx, dbg!(self.y()) - dbg!(dy), dbg!(self.z()) - dbg!(dz)));

        let mut graphemes = UnicodeSegmentation::graphemes(&input[..offset - dz], true);

        if "\n" == next.current(&input) {
            match graphemes.next() {
                Some(grapheme) => {
                    next.2 -= grapheme.len();
                    //next.1 += 1;
                    next.0 -= grapheme.len();
                }
                None => {}
            };
        }

        next
    }
    */

    pub fn backward_graphemes(&self, input: &str, count: usize) -> Self {
        let next = input
            .get(..self.z())
            .iter()
            .flat_map(|slice| slice.graphemes(true))
            .rev()
            .fold_while(self.clone(), |mut next, grapheme| {
                next.2 -= grapheme.len();

                if "\n" == grapheme {
                    next.1 -= 1;
                } else {
                    next.0 -= grapheme.len();
                }

                if self.z() - next.z() >= count {
                    if "\n" == grapheme {
                        Continue(next)
                    } else {
                        Done(next)
                    }
                } else {
                    Continue(next)
                }
            })
            .into_inner();

        next

        /*
        let mut next = input
            .get(..self.z())
            .iter()
            .flat_map(|slice| slice.graphemes(true))
            .rev()
            .take(count)
            .fold(self.clone(), |cursor, grapheme| {
                let mut next = cursor;

                next.2 -= grapheme.len();

                if "\n" == grapheme {
                    next.1 -= 1;
                } else {
                    next.0 -= grapheme.len();
                }

                next
            });

        if self.y() != next.y() {
            let to_previous_line = input
                .get(..next.z())
                .iter()
                .flat_map(|slice| slice.graphemes(true))
                .rev()
                .take_while(|grapheme| dbg!(*grapheme) != "\n")
                .fold(0, |to_previous_line, grapheme| {
                    to_previous_line + grapheme.len()
                });

            next.0 = to_previous_line;
        }

        if next.current(input) == "\n" {
            if let Some(grapheme) = input
                .get(..next.z())
                .iter()
                .flat_map(|slice| slice.graphemes(true))
                .next_back()
            {
                next.2 -= grapheme.len();
                next.0 -= grapheme.len();
            }
        }

        //next
        */
    }
}

#[test]
fn backward_graphemes() {
    let buffer = include_str!("../edit");

    macro_rules! assert_ {
        ($from:expr, $steps:expr,  $to:expr, $want:expr) => {
            let from: Cursor = $from.into();
            let to = from.backward_graphemes(buffer, $steps);
            let got = to.current(buffer);
            assert_eq!(
                to,
                $to.into(),
                "{:?} - {}g = got {:?} {:?} want {:?} {:?}",
                $from,
                $steps,
                to,
                got,
                $to,
                $want
            );
            assert_eq!(
                got, $want,
                "{:?} - {}g = got {:?} want {:?}",
                $from, $steps, got, $want
            );
        };
    }

    let tests = vec![
        ((1, 0, 1), 1, (0, 0, 0), "#"),
        ((2, 0, 2), 1, (1, 0, 1), " "),
        ((3, 0, 3), 1, (2, 0, 2), "J"),
        ((4, 0, 4), 1, (3, 0, 3), "a"),
        ((5, 0, 5), 1, (4, 0, 4), "g"),
        ((0, 1, 7), 1, (5, 0, 5), "o"),
        ((1, 0, 1), 1, (0, 0, 0), "#"),
        ((3, 0, 3), 2, (1, 0, 1), " "),
        ((0, 1, 7), 3, (3, 0, 3), "a"),
        ((3, 2, 11), 4, (5, 0, 5), "o"),
        ((8, 2, 15), 5, (3, 2, 10), "C"),
    ];

    for (from, steps, to, want) in tests {
        println!("{:?} - {} = {:?} {:?}", from, steps, to, want);
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
