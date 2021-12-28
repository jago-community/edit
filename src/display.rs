use std::{
    env::current_dir,
    fs::File,
    io::{stdout, Read, Write},
    iter::Peekable,
};

use crossterm::{
    cursor::{CursorShape, MoveTo, RestorePosition, SavePosition, SetCursorShape},
    event::{read, Event, KeyCode, KeyEvent},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{buffer::Buffer, document::Document};

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

    let mut document = Document::new(&source)?;
    let mut buffer = Buffer::new(&source);

    let mut output = stdout();

    execute!(
        output,
        EnterAlternateScreen,
        SetCursorShape(CursorShape::UnderScore),
        &buffer,
    )?;

    enable_raw_mode()?;

    loop {
        disable_raw_mode()?;

        let position = crossterm::cursor::position()?;

        document.focus(position);

        //execute!(output, &document)?;
        //execute!(output, &buffer)?;

        enable_raw_mode()?;

        let event = read()?;

        buffer.handle(&event);

        //if let Err(error) = document.handle(event.clone(), &output) {
        //document.caught(error);
        //}

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => {
                println!("break");
                break;
            }
            _ => {}
        };

        queue!(output, &buffer)?;

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
    #[error("Document {0}")]
    Document(#[from] crate::document::Error),
    #[error("Buffer")]
    Buffer,
}
