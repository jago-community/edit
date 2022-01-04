pub struct Cursor {
    position: usize,
    //coordinates: (usize, usize),
}

impl Cursor {
    pub fn new(position: usize) -> Cursor {
        Cursor {
            position,
            //coordinates,
        }
    }
}
