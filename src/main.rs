use std::io::{stdin, stdout};

use jom::{handle_event, render, App, Distributions, Result, Screen, Terminal, Tui};
use termion::{input::TermRead, raw::IntoRawMode};

const DISTROS: &str = include_str!("distros_packages.ron");

fn main() -> Result<()> {
    let distros: Distributions = ron::from_str(DISTROS)?;
    let mut app = App::new(distros);

    let stdout = stdout().into_raw_mode()?;
    let terminal = Terminal::new(stdout);
    let mut tui = Tui::new(terminal);

    let mut keys = stdin().keys();

    tui.init();
    while tui.screen != Screen::None {
        tui.draw(|t| render(t, &app));
        let key = keys.next().unwrap()?;
        handle_event(key, &mut tui, &mut app);
    }
    tui.exit();

    Ok(())
}
