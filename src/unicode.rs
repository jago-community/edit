use itertools::Itertools;
use unicode_segmentation::UnicodeSegmentation;

pub fn split_line_bounds<'a>(buffer: &'a str) -> impl Iterator<Item = &'a str> {
    UnicodeSegmentation::split_word_bound_indices(buffer)
        .peekable()
        .batching(|rest| {
            let (start, this) = rest.next()?;
            let mut stop = start + this.len();

            if this == "\n" {
                return Some(&buffer[start..stop]);
            }

            while let Some((index, next)) = rest.peek() {
                if *next == "\n" {
                    break;
                }

                stop = *index + next.len();

                drop(rest.next());
            }

            Some(&buffer[start..stop])
        })
}

#[test]
fn line_bounds() {
    let buffer = include_str!("../edit");

    let mut line_bounds = split_line_bounds(buffer);

    assert_eq!(line_bounds.next(), Some("# Jago"));
    assert_eq!(line_bounds.next(), Some("\n"));
    assert_eq!(line_bounds.next(), Some("\n"));
    assert_eq!(line_bounds.next(), Some("> `Canker` but communist."));
    assert_eq!(line_bounds.next(), Some("\n"));
    assert_eq!(line_bounds.next(), Some("\n"));
    assert_eq!(line_bounds.next(), Some("## Intro"));
    assert_eq!(line_bounds.next(), Some("\n"));
    assert_eq!(line_bounds.next(), Some("\n"));
    assert_eq!(line_bounds.next(), Some("The name Alec Thompson is one that most of us know for one reason or another. The same face might come to mind for each of us."));
    assert_eq!(line_bounds.next(), Some("\n"));
    assert_eq!(line_bounds.next(), Some("\n"));
    assert_eq!(line_bounds.next(), Some("## Canker"));
    assert_eq!(line_bounds.next(), Some("\n"));
    assert_eq!(line_bounds.next(), Some("\n"));
    assert_eq!(
        line_bounds.next(),
        Some("Canker was founded by one of the Alec Thompsons.")
    );
    assert_eq!(line_bounds.next(), Some("\n"));
}
