#![feature(core_intrinsics)]

mod event;
mod util;

use crate::event::{Event, Events};
use std::intrinsics::breakpoint;
use std::{
    error::Error, fs::read_to_string, fs::File, io::stdout, io::Write, path::PathBuf, sync::Once,
};
use structopt::{clap::AppSettings, StructOpt};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, style::*, widgets::*, Terminal};
use unicode_width::UnicodeWidthStr;

#[derive(StructOpt)]
#[structopt(setting = AppSettings::InferSubcommands)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: PathBuf,
}

struct App {
    input: String,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::from_args();
    let read = read_to_string(&args.path);
    let path = args.path.into_os_string().into_string().unwrap();
    let mut app = App::default();
    let events = Events::new();
    let mut terminal = Terminal::new(TermionBackend::new(AlternateScreen::from(
        MouseTerminal::from(stdout().into_raw_mode()?),
    )))?;
    static SYNC_OBJ: Once = Once::new();

    if read.is_err() {
        println!("File could not be read")
    }

    let read = read.unwrap().trim_end().to_string();

    loop {
        SYNC_OBJ.call_once(|| {
            terminal.clear().unwrap();
            for line in read.lines() {
                app.input.push_str(&*format!("{}\n", line));
            }
            app.input.push('\n');
            app.input.pop();
        });

        terminal.draw(|f| {
            if let Event::Input(key) = events.next().unwrap() {
                match key {
                    Key::Ctrl('x') => unsafe { breakpoint() },
                    Key::Ctrl('s') => File::create(&path)
                        .unwrap()
                        .write_all(app.input.as_bytes())
                        .unwrap(),
                    Key::Char('\n') => {
                        app.input.push('\n');
                    }
                    Key::Char(c) => {
                        app.input.push(c);
                    }
                    Key::Backspace => {
                        app.input.pop();
                    }
                    Key::Down => {}
                    _ => {}
                }
            }

            let input = Paragraph::new(app.input.as_ref())
                .style(Style::default())
                .block(Block::default().borders(Borders::ALL).title(&*path));
            f.render_widget(input, f.size());

            f.set_cursor(
                f.size().x + app.input.width() as u16 + 1,
                f.size().y + read.lines().count() as u16,
            )
        })?;
    }
}
