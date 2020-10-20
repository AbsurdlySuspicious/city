use crate::city::City;
use std::io::StdoutLock;
use std::fmt::Write as fmtWrite;
use std::io::Write as ioWrite;

pub const SIZE_DEFAULT_W: usize = 150;
pub const SIZE_DEFAULT_H: usize = 40;
pub const SIZE_MIN_W: usize = 30;
pub const SIZE_MIN_H: usize = 30;
pub const SIZE_AUTO_PAD_W: usize = 0;
pub const SIZE_AUTO_PAD_H: usize = 5;

pub fn get_term_size() -> (usize, usize) {
    match term_size::dimensions() {
        Some((w, h)) => (w.saturating_sub(SIZE_AUTO_PAD_W), h.saturating_sub(SIZE_AUTO_PAD_H)),
        None => panic!("Can't get terminal size, try removing -a"),
    }
}

pub fn setup_console() {
    //print!("\x1b[?1049h\x1b[1;1H\x1b[?25l"); // go to alt buffer and disable cursor
    print!("\x1b[?25l") // disable cursor
}

pub fn prepare_canvas(height: usize) {
    println!("info line");
    for _ in 0..height {
        println!();
    }
}

pub fn destroy_console() {
    //println!("\x1b[?25h\x1b[?1049l"); // enable cursor and go to normal buffer
    println!("\x1b[?25h"); // enable cursor
}

pub fn draw_to_console(c: &City, buf: &mut String, out: &mut StdoutLock) {
    buf.clear();
    let (_, height) = c.get_size();

    // move up to beginning, clear first (info) line and clear styles
    write!(buf, "\x1b[0m\x1b[{}A\x1b[2K\r", height + 1).unwrap();
    write!(buf, "tick: {}", c.get_tick()).unwrap();

    let canvas = c.get_canvas();
    for row in canvas.row_iter() {
        buf.push('\n');
        for &color in row {
            write!(buf, "\x1b[{}m ", color).unwrap();
        }
    }

    writeln!(out, "{}", buf).unwrap();
}
