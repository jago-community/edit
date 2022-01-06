#[derive(Debug, PartialEq)]
pub struct Cursor(usize, usize, usize);

impl Cursor {
    fn z(&self) -> usize {
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
    fn forward(&mut self, buffer: &str, x: usize, y: usize) {
        let mut words = UnicodeSegmentation::split_word_bound_indices(&buffer[self.z()..]);

        let mut dz = 0;
        let mut dy = 0;

        for _ in 0..y {
            if let Some((index, next)) = words.find(|(_, this)| *this == "\n") {
                dz = index + next.len();
                dy += next.len();
            } else {
                break;
            }

            if dy == y {
                break;
            }
        }

        let mut graphemes = UnicodeSegmentation::grapheme_indices(&buffer[self.z() + dz..], true);

        let mut dx = 0;

        dbg!((dx, dy, dz));

        for fx in 0..self.0 + x {
            match graphemes.next() {
                Some((_, "\n")) => break,
                Some((_, next)) => {
                    dz += next.len();

                    if fx >= self.0 {
                        dx += next.len();
                    }
                }
                _ => break,
            };
        }

        self.0 += dx;
        self.1 += dy;
        self.2 += dz;
    }

    fn current<'a>(&self, buffer: &'a str) -> &'a str {
        let mut buffer = UnicodeSegmentation::graphemes(&buffer[self.z()..], true);

        match buffer.next() {
            Some(next) => next,
            None => "",
        }
    }
}

#[test]
fn forward() {
    let buffer = include_str!("../edit");

    macro_rules! assert_cursor {
        ($got:expr,  $want:expr, $slice:expr) => {
            let current = $got.current(buffer);
            assert_eq!($got, $want.into(), "got {:?} want {:?}", $got, $want);
            assert_eq!(current, $slice, "got {:?} want {:?}", current, $slice);
        };
    }

    let mut cursor: Cursor = (0, 0, 0).into();

    assert_cursor!(cursor, (0, 0, 0), "#");

    cursor.forward(buffer, 1, 0);

    assert_cursor!(cursor, (1, 0, 1), " ");

    cursor.forward(buffer, 2, 2);

    assert_cursor!(cursor, (3, 2, 11), "C");
}

impl Cursor {
    fn seek(&mut self, buffer: &str, (x, y): (usize, usize)) {
        let mut words = UnicodeSegmentation::split_word_bound_indices(&buffer[self.z()..]);

        let mut dz = 0;
        let mut dy = 0;

        for _ in self.1..y {
            if let Some((index, next)) = words.find(|(_, this)| *this == "\n") {
                dz = index + next.len();
                dy += next.len();
            } else {
                break;
            }

            if dy == y {
                break;
            }
        }

        let mut graphemes = UnicodeSegmentation::grapheme_indices(&buffer[self.z() + dz..], true);

        let mut dx = 0;

        dbg!((dx, dy, dz));

        for _ in 0..x {
            match graphemes.next() {
                Some((_, "\n")) => break,
                Some((_, next)) => {
                    dz += next.len();
                    dx += next.len();
                }
                _ => break,
            };
        }

        self.0 = dx;
        self.1 = dy;
        self.2 = dz;
    }
}

#[test]
fn seek() {
    let buffer = include_str!("../edit");

    macro_rules! assert_cursor {
        ($got:expr,  $want:expr, $slice:expr) => {
            let current = $got.current(buffer);
            assert_eq!($got, $want.into(), "got {:?} want {:?}", $got, $want);
            assert_eq!(current, $slice, "got {:?} want {:?}", current, $slice);
        };
    }

    let mut cursor: Cursor = (0, 0, 0).into();

    assert_cursor!(cursor, (0, 0, 0), "#");

    cursor.seek(buffer, (1, 0));

    assert_cursor!(cursor, (1, 0, 1), " ");

    cursor.seek(buffer, (3, 2));

    assert_cursor!(cursor, (3, 2, 11), "C");

    cursor.seek(buffer, (1, 4));

    assert_cursor!(cursor, (1, 4, 37), "#");
}
