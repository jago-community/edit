use std::{
    env::current_dir,
    fs::File,
    io::{stdout, Read, Write},
    iter::Peekable,
};

use crossterm::{
    cursor::{CursorShape, SetCursorShape},
    event::{read, Event, KeyCode, KeyEvent},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::document::Document;

pub fn handle(_: &mut Peekable<impl Iterator<Item = String>>) -> Result<(), Error> {
    let mut source = vec![];

    let target = current_dir().map_err(Error::from).and_then(|directory| {
        directory
            .file_stem()
            .ok_or_else(|| Error::Incomplete)
            .map(|file_stem| directory.join(file_stem))
    })?;

    let mut file = File::open(target)?;

    file.read_to_end(&mut source)?;

    let mut document = Document::new(&source);

    let mut output = stdout();

    execute!(
        output,
        EnterAlternateScreen,
        SetCursorShape(CursorShape::UnderScore),
        &document,
    )?;

    enable_raw_mode()?;

    loop {
        disable_raw_mode()?;

        let position = crossterm::cursor::position()?;

        execute!(output, &document)?;

        enable_raw_mode()?;

        let event = read()?;

        document.handle(&event);

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => {
                break;
            }
            _ => {}
        };

        queue!(output, &document)?;

        output.flush()?;
    }

    disable_raw_mode()?;

    execute!(
        output,
        SetCursorShape(CursorShape::Block),
        LeaveAlternateScreen
    )?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
    //#[error("Document {0}")]
    //Document(#[from] crate::document::Error),
    #[error("Buffer")]
    Buffer,
}
