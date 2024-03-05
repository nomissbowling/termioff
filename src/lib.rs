#![doc(html_root_url = "https://docs.rs/termioff/0.1.0")]
//! terminal utilities for Rust with termion
//!

use std::fmt;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::thread;
use std::sync::mpsc;

use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::input::{TermRead, MouseTerminal};
use termion::event::Event;
use termion::{terminal_size, cursor, clear, color, style};

/// tuple TRX for mpsc::channel Result termion Event (not std::error::Error)
pub type TplTRX = (
  mpsc::Sender<Result<Event, std::io::Error>>,
  mpsc::Receiver<Result<Event, std::io::Error>>);

/// trait So
pub trait So: Write {
  /// begin cursor::Hide clear::All
  fn begin(&mut self) -> Result<(), Box<dyn Error>> {
    write!(self, "{}{}", cursor::Hide, clear::All)?;
    self.flush()?;
    Ok(())
  }

  /// fin cursor::Show
  fn fin(&mut self) -> Result<(), Box<dyn Error>> {
    write!(self, "{}", cursor::Show)?;
    self.flush()?;
    Ok(())
  }

  /// style
  fn style<T: fmt::Display>(&mut self, s: T) -> Result<(), Box<dyn Error>> {
    write!(self, "{}", s)?;
    Ok(())
  }

  /// write
  fn wr<BGC: color::Color, FGC: color::Color>(&mut self, x: u16, y: u16,
    st: u16, bg: BGC, fg: FGC, msg: &String) -> Result<(), Box<dyn Error>> {
    let styles: Vec<Box<dyn fmt::Display>> = vec![
      Box::new(style::Bold), Box::new(style::Italic)];
    for (i, s) in styles.iter().enumerate() {
      if st & 2^(i as u16) != 0 { self.style(s)?; }
    }
    write!(self, "{}{}{}{}{}",
      cursor::Goto(x, y), color::Bg(bg), color::Fg(fg), msg, style::Reset)?;
    self.flush()?;
    Ok(())
  }
}

/// RawTerminal
impl So for RawTerminal<std::io::Stdout> {
}

/// AlternateScreen RawTerminal
impl So for AlternateScreen<RawTerminal<std::io::Stdout>> {
}

/// MouseTerminal RawTerminal
impl So for MouseTerminal<RawTerminal<std::io::Stdout>> {
}

/// MouseTerminal AlternateScreen RawTerminal
impl So for MouseTerminal<AlternateScreen<RawTerminal<std::io::Stdout>>> {
}

/// AlternateScreen MouseTerminal RawTerminal
impl So for AlternateScreen<MouseTerminal<RawTerminal<std::io::Stdout>>> {
}

/// Termioff
// #[derive(Debug)]
pub struct Termioff {
  /// width
  pub w: u16,
  /// height
  pub h: u16,
  /// so stdout
  pub so: Box<dyn Write>
}

/// Debug
impl fmt::Debug for Termioff {
  /// fmt
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {}) [stdout]", self.w, self.h)
  }
}

/// Display
impl fmt::Display for Termioff {
  /// fmt
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

/// Termioff
impl Termioff {
  /// constructor
  pub fn new(k: u16) -> Result<Self, Box<dyn Error>> {
    let (w, h) = terminal_size()?;
    let so: RawTerminal<std::io::Stdout> = stdout().into_raw_mode()?;
    let so: Box<dyn Write> = match k {
    4 => Box::new(AlternateScreen::from(MouseTerminal::from(so))),
    3 => Box::new(MouseTerminal::from(AlternateScreen::from(so))),
    2 => Box::new(MouseTerminal::from(so)),
    1 => Box::new(AlternateScreen::from(so)),
    _ => Box::new(so)
    };
    Ok(Termioff{w, h, so})
  }

  /// begin cursor::Hide clear::All
  pub fn begin(&mut self) -> Result<(), Box<dyn Error>> {
    write!(self.so, "{}{}", cursor::Hide, clear::All)?;
    self.so.flush()?;
    Ok(())
  }

  /// fin cursor::Show
  pub fn fin(&mut self) -> Result<(), Box<dyn Error>> {
    write!(self.so, "{}", cursor::Show)?;
    self.so.flush()?;
    Ok(())
  }

  /// style
  pub fn style<T: fmt::Display>(&mut self, s: T) -> Result<(), Box<dyn Error>> {
    write!(self.so, "{}", s)?;
    // self.so.flush()?;
    Ok(())
  }

  /// write
  pub fn wr<BGC: color::Color, FGC: color::Color>(&mut self, x: u16, y: u16,
    st: u16, bg: BGC, fg: FGC, msg: &String) -> Result<(), Box<dyn Error>> {
    let styles: Vec<Box<dyn fmt::Display>> = vec![
      Box::new(style::Bold), Box::new(style::Italic)];
    for (i, s) in styles.iter().enumerate() {
      if st & 2^(i as u16) != 0 { self.style(s)?; }
    }
    write!(self.so, "{}{}{}{}{}",
      cursor::Goto(x, y), color::Bg(bg), color::Fg(fg), msg, style::Reset)?;
    self.so.flush()?;
    Ok(())
  }

  /// prepare thread
  pub fn prepare_thread(&self) -> Result<TplTRX, Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();
    if true { // closure once
      let si = stdin();
      let tx = tx.clone();
      let _handle = thread::spawn(move || { // for non blocking to fetch event
        for ev in si.events() { tx.send(ev).expect("send"); } // loop forever
        () // will not be disconnected
      });
    }
    Ok((tx, rx))
  }
}

/// test with [-- --nocapture] or [-- --show-output]
#[cfg(test)]
mod tests {
  use super::Termioff;
  use termion::color::Rgb;

  /// test a
  #[test]
  fn test_a() {
    let s = String::from_utf8("ABC".into()).expect("utf8");
    let mut tm = Termioff::new(2).expect("construct");
    tm.begin().expect("begin");
    tm.wr(1, 50, 3, Rgb(255, 255, 255), Rgb(0, 0, 0), &s).expect("wr");
    tm.fin().expect("fin");
    assert_eq!(tm.w, 80);
    assert_eq!(tm.h, 50);
  }
}
