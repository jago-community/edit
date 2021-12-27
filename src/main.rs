mod buffer;
mod display;
mod document;

fn main() {
    let mut input = std::env::args().skip(1).peekable();

    display::handle(&mut input);
}
