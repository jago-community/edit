mod buffer;
//mod context;
mod cursor;
mod display;
mod document;
mod plane;

fn main() {
    let mut input = std::env::args().skip(1).peekable();

    display::handle(&mut input).unwrap();
}
