#![allow(unused)]
use std::collections::HashSet;
use std::io::{Stdout, Write};

use serde::Deserialize;
use termion::{clear, cursor, event::Key, raw::RawTerminal};

mod prelude;
pub use prelude::Result;

#[derive(Deserialize, Debug, Default)]
struct Package {
    name: String,
    /// Commands for package installation.
    install: Vec<String>,
    /// Commands for package uninstallation.
    uninstall: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Distro {
    name: String,
    packages: Vec<Package>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Distributions(Vec<Distro>);
impl Distributions {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    fn get_distro(&self, distro: usize) -> String {
        self.0.get(distro).unwrap().name.clone()
    }
}

pub struct Terminal {
    stdout: RawTerminal<Stdout>,
    /// Cursor position.
    cp: usize,
}

impl Terminal {
    pub fn new(stdout: RawTerminal<Stdout>) -> Self {
        Self { stdout, cp: 0 }
    }

    pub fn as_ref<'a, 'b: 'a>(&'b mut self) -> &mut Self {
        self
    }

    pub fn show_cursor(&mut self) {
        self.print(cursor::Show);
    }

    pub fn hide_cursor(&mut self) {
        self.print(cursor::Hide);
    }

    pub fn clear(&mut self) {
        self.print(clear::All);
    }

    pub fn move_cursor(&mut self, x: u16, y: u16) {
        self.print(cursor::Goto(x, y))
    }

    pub fn print(&mut self, text: impl std::fmt::Display) {
        write!(self.stdout, "{}", text).unwrap();
    }

    pub fn flush(&mut self) {
        self.stdout.flush().unwrap();
    }
}

#[derive(PartialEq)]
pub enum Screen {
    None = 0,
    One,
    Two,
    Three,
}

impl Screen {
    fn from_u8(num: u8) -> Self {
        match num {
            0 => Screen::None,
            1 => Screen::One,
            2 => Screen::Two,
            3 => Screen::Three,
            e => panic!("invalid screen {}", e),
        }
    }

    fn to_u8(&self) -> u8 {
        match self {
            Screen::None => 0,
            Screen::One => 1,
            Screen::Two => 2,
            Screen::Three => 3,
        }
    }
}

pub struct Tui {
    pub terminal: Terminal,
    pub screen: Screen,
}

impl Tui {
    pub fn new(terminal: Terminal) -> Self {
        Self { terminal, screen: Screen::One }
    }

    pub fn draw<'a, 'b: 'a, A>(&'b mut self, view: A)
    where
        A: Fn(&mut Self),
    {
        view(self)
    }

    pub fn init(&mut self) {
        self.terminal.clear();
    }

    pub fn exit(&mut self) {
        self.terminal.show_cursor();
        self.terminal.print(format!("{}[2J", 27 as char));
        self.terminal.clear();
    }
}

pub fn render(t: &mut Tui, app: &App) {
    match &t.screen {
        Screen::One => screen_one(&mut t.terminal, app),
        Screen::Two => screen_two(&mut t.terminal, app),
        Screen::Three => {}
        Screen::None => {}
    }
}

pub fn screen_one(t: &mut Terminal, app: &App) {
    t.hide_cursor();
    t.move_cursor(1, 2);
    t.print("What would you like to do?. \r\n");

    let distros = &app.distros.0;

    for (i, distro) in distros.iter().enumerate() {
        let arrow = if t.cp == i { "> " } else { "  " };
        t.print(arrow);
        t.print(format!("{}\r\n", distro.name));
    }

    t.print("To exit, type Q, ESC or Ctrl + C. \r\n ");
    t.flush();
}

pub fn screen_two(t: &mut Terminal, _app: &App) {
    let items = ["install", "uninstall"];
    t.hide_cursor();
    t.move_cursor(1, 2);
    t.print("What would you like to do?. \r\n");

    for (i, item) in items.iter().enumerate() {
        let arrow = if t.cp == i { "> " } else { "  " };
        t.print(arrow);
        t.print(format!("{}\r\n", item));
    }

    t.print("To exit, type Q, ESC or Ctrl + C. \r\n ");
    t.flush();
}

pub fn handle_event(key: Key, tui: &mut Tui, app: &mut App) {
    let Tui { screen, terminal } = tui;
    let cp = &mut terminal.cp;

    let (new_screen, offset) = match (&screen, key) {
        (Screen::One, Key::Up) => (1, -1),
        (Screen::One, Key::Esc) => (0, 0),
        (Screen::One, Key::Down) => (1, 1),
        (Screen::One, Key::Ctrl('c')) => (0, 0),
        (Screen::One, Key::Char('q')) => (0, 0),
        (Screen::One, Key::Char('\n')) => (2, 0),
        (Screen::Two, Key::Backspace) => todo!(),
        (Screen::Two, Key::Up) => todo!(),
        (Screen::Two, Key::Down) => todo!(),
        (Screen::Two, Key::Char('q')) => (0, 0),
        (Screen::Two, Key::Ctrl('c')) => (0, 0),
        (Screen::Three, Key::Char('\n')) => (3, 0),
        (Screen::Two, Key::Esc) => (0, 0),
        // this line below keep the things
        _ => (screen.to_u8(), 0),
    };

    let dlength = app.distros.0.len();
    *cp = clamp_add(dlength, *cp, offset);
    *screen = Screen::from_u8(new_screen);
    app.selected_distro = *cp;
}

fn clamp_add(distros_len: usize, curr_cp: usize, offset: isize) -> usize {
    // lp: Latest package. This is the index of latest distro.
    let lp = distros_len as isize - 1;
    let new_cp = curr_cp as isize + offset;

    let clamped_cp = match new_cp {
        i if i >= lp => lp,
        i if i <= 0 => 0,
        i => i,
    };
    clamped_cp as usize
}

#[derive(Debug, Default)]
pub struct App {
    distros: Distributions,
    selected_distro: usize,
    selected_packages: HashSet<String>,
    pub is_running: bool,
}
impl App {
    pub fn new(distros: Distributions) -> Self {
        Self { distros, is_running: true, ..Default::default() }
    }

    pub fn set_distro(&mut self, index: usize) {
        self.selected_distro = index;
    }

    pub fn set_package(&mut self, name: String) {
        self.selected_packages.insert(name);
    }

    pub fn get_distro_name(&self, distro: usize) -> String {
        self.distros.0.get(distro).unwrap().name.clone()
    }
}
