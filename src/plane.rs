pub struct Plane<'a> {
    points: &'a [Point],
}

struct Point {
    index: usize,
    value: u8,
}

#[test]
fn test_plane() {
    let input = b"# Jago

> `Canker` but communist.

## Intro

The name Alec Thompson is one that most of us know for one reason or another. The same face might come to mind for each of us. 

## Canker

Canker was founded by one of the Alec Thompsons.
";
}
