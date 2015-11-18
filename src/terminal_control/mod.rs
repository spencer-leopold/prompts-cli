#![feature(libc)]
extern crate libc;

// Linux specifc termios structure definition
//
// Since we don't actually access any of the fields individually, and instead just
// pass around termios as a "black box", this will probably work for other platforms
// as long their struct termios is smaller than Linux's. For example, Mac OS omits the
// c_line field and only has 20 control characters.
#[allow(non_camel_case_types)]
struct termios {
    c_iflag:  libc::c_uint,          // input mode flags
    c_oflag:  libc::c_uint,          // output mode flags
    c_cflag:  libc::c_uint,          // control mode flags
    c_lflag:  libc::c_uint,          // local mode flags
    c_line:   libc::c_uchar,         // line discipline
    c_cc:     [libc::c_uchar; 32], // control characters
    c_ispeed: libc::c_uint,          // input speed
    c_ospeed: libc::c_uint,          // output speed
}

extern {
    fn tcgetattr(filedes: libc::c_int, termptr: *mut termios) -> libc::c_int;
    fn tcsetattr(filedes: libc::c_int, opt: libc::c_int, termptr: *const termios) -> libc::c_int;
    fn cfmakeraw(termptr: *mut termios);
}

fn get_terminal_attr() -> (termios, libc::c_int) {
    unsafe {
      let mut ios = termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc:    [0; 32],
        c_ispeed: 0,
        c_ospeed: 0
      };
      // first parameter is file descriptor number, 0 ==> standard input
      let err = tcgetattr(0, &mut ios);
      return (ios, err);
    }
}

use std::mem;
fn make_raw<'a>(ios: *mut termios) -> &'a termios {
    unsafe {
      let mut ios = mem::transmute(Box::new(ios));
      // let mut ios = *ios;
      cfmakeraw(ios);
      return &*ios;
    }
}

fn set_terminal_attr(ios: &termios) -> libc::c_int {
    unsafe {
      // first paramter is file descriptor number, 0 ==> standard input
      // second paramter is when to set, 0 ==> now
      return tcsetattr(0, 0, ios);
    }
}

pub struct TerminalRestorer {
    ios: termios
}

impl Drop for TerminalRestorer {
    fn drop(&mut self) {
        set_terminal_attr(&self.ios);
    }
}

pub fn set_terminal_raw_mode() -> TerminalRestorer {
    let (mut original_ios, err) = get_terminal_attr();
    if err != 0 {
        panic!("failed to get terminal settings");
    }

    let raw_ios = make_raw(&mut original_ios);
    let err = set_terminal_attr(&raw_ios);
    if err != 0 {
        panic!("failed to switch terminal to raw mode");
    }

    TerminalRestorer {
        ios: original_ios
    }
}

pub fn reset_terminal(restorer: TerminalRestorer) {
    let err = set_terminal_attr(&restorer.ios);
    if err != 0 {
        panic!("failed to switch terminal to back");
    }
}
