extern crate libc;
extern crate term;

use std::io;
use std::str;
use std::process;
use std::io::BufReader;
use std::io::prelude::*;
use std::mem::transmute;
use std::collections::LinkedList;
use std::collections::HashSet;

mod terminal_control;

pub struct OptionStore {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
enum ReadResult {
    Up,
    Down,
    Right,
    Left,
    Space,
    Enter,
    CtrlC,
    CtrlD,
    Other
}

extern {
    fn read(fd: libc::c_int, buf: *mut u8, nbyte: u64) -> i64;
}

fn read_stdin() -> ReadResult {
    unsafe {
        //
        // Reading bytes into storage for an unsigned integer
        //
        let mut buf = 0u64;
        let bufAddr: *mut u8 = transmute(&mut buf);

        // first parameter is file descriptor number
        let numRead = read(0, bufAddr, 8);

        if numRead < 0 {
            println!("error reading standard input");
        }

        match buf {
            0x415B1B => ReadResult::Up,
            0x425B1B => ReadResult::Down,
            0x435B1B => ReadResult::Right,
            0x445B1B => ReadResult::Left,
            0x0d => ReadResult::Enter,
            0x20 => ReadResult::Space,
            0x04 => ReadResult::CtrlD,
            0x03 => ReadResult::CtrlC,
            _ => ReadResult::Other
        }
    }
}

pub fn yes_or_no(question: String, prompt: &'static str, default: String) -> String {
    print!("{} ", question);
    io::stdout().flush().unwrap();

    let r_value: String;
    let mut r_value_res = String::new();


    io::stdin().read_line(&mut r_value_res).ok().expect("failed to read shortname");

    if r_value_res.as_bytes()[0] == b'y' {
        r_value = default;
    }
    else {
        let mut r_value_custom = String::new();
        print!("{} ", prompt);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut r_value_custom).ok().expect("failed to read shortname");
        r_value = r_value_custom;
    }

    str::replace(&r_value, "\n", "")
}

pub fn list(options: LinkedList<OptionStore>) -> String {

    let _restore = terminal_control::set_terminal_raw_mode();

    print!("{}[", '\x1B');
    print!("?25l");

    let mut t = term::stdout().unwrap();

    let mut pos = 1;

    let total = options.len();
    
    let mut idx = 1;

    for option in &options {
        if idx == 1 {
            t.fg(term::color::GREEN).unwrap();
            print!("{} ", '\u{29BE}');
            t.reset().unwrap();
            print!("{}\r\n", option.key);
        }
        else {
            print!("{} {}\r\n", " ".to_owned(), option.key);
        }

        idx += 1;
    }

    loop {
        io::stdout().flush().unwrap();

        match read_stdin() {
            ReadResult::Enter => {

                let mut answer: String = String::new();
                let mut start_at = 1;

                for option in &options {
                    if start_at == pos {
                        answer = option.value.clone();
                        break;
                    }

                    start_at += 1;
                }

                print!("{}[", '\x1B');
                print!("?25h");
                return answer.to_owned();
            },
            input => {
                match input {
                    ReadResult::Up => {
                        if pos <= 1 {
                            pos = total;
                        }
                        else {
                            pos -= 1;
                        }
                    },
                    _ => {
                        if pos >= total {
                            pos = 1;
                        }
                        else {
                            pos += 1;
                        }
                    }
                }

                // // Erases current line
                // print!("{}[", '\x1B');
                // print!("1K");

                // Moves cursor to previos 'n' line
                print!("{}[", '\x1B');
                print!("{}F", total); 

                // Moves cursor to beginning line
                print!("{}[", '\x1B');
                print!("1G");

                let mut current_idx = 1;
                for option in &options {
                    if current_idx == pos {
                        t.fg(term::color::GREEN).unwrap();
                        print!("{} ", '\u{29BE}');
                        t.reset().unwrap();
                        print!("{}\r\n", option.key);
                    }
                    else {
                        print!("{} {}\r\n", " ".to_owned(), option.key);
                    }

                    current_idx += 1;
                }
            }
        }
    }

    // Show Cursor
    print!("{}[", '\x1B');
    print!("?25h");
    t.reset().unwrap();
    String::from("")
}

pub fn checkboxes(options: LinkedList<OptionStore>) -> String {

    let _restore = terminal_control::set_terminal_raw_mode();

    // Hide Cursor
    print!("{}[", '\x1B');
    print!("?25l");

    let mut t = term::stdout().unwrap();

    let mut pos = 1;

    let total = options.len();
    
    let mut idx = 1;

    for option in &options {
        if idx == 1 {
            t.fg(term::color::GREEN).unwrap();
            print!("{} ", '\u{29BE}');
            t.reset().unwrap();
            print!("{}\r\n", option.key);
        }
        else {
            print!("{} ", '\u{29BE}');
            print!("{}\r\n", option.key);
        }

        idx += 1;
    }

    let mut selected = HashSet::new();

    loop {
        io::stdout().flush().unwrap();

        match read_stdin() {
            ReadResult::Enter => {
                print!("{}[", '\x1B');
                print!("?25h");
                return "results".to_owned();
            },
            ReadResult::Space => {
                if selected.contains(&pos) {
                    selected.remove(&pos);
                }
                else {
                    selected.insert(pos.clone());
                }

                // Moves cursor to previous 'n' line
                print!("{}[", '\x1B');
                print!("{}F", total); 

                // Moves cursor to beginning line
                print!("{}[", '\x1B');
                print!("1G");

                let mut current_idx = 1;
                for option in &options {
                    if selected.contains(&current_idx) {
                        t.fg(term::color::GREEN).unwrap();
                        print!("{} ", '\u{29BF}');
                        t.reset().unwrap();
                        print!("{}\r\n", option.key);
                    }
                    else if current_idx == pos {
                        t.fg(term::color::GREEN).unwrap();
                        print!("{} ", '\u{29BE}');
                        t.reset().unwrap();
                        print!("{}\r\n", option.key);
                    }
                    else {
                        print!("{} ", '\u{29BE}');
                        print!("{}\r\n", option.key);
                    }

                    current_idx += 1;
                }
            },
            input => {
                match input {
                    ReadResult::CtrlD => {
                        // Show Cursor
                        print!("{}[", '\x1B');
                        print!("?25h");
                        io::stdout().flush().unwrap();
                        terminal_control::reset_terminal(_restore);
                        process::exit(1);
                    },
                    ReadResult::CtrlC => {
                        // Show Cursor
                        print!("{}[", '\x1B');
                        print!("?25h");
                        io::stdout().flush().unwrap();
                        terminal_control::reset_terminal(_restore);
                        process::exit(1);
                    },
                    ReadResult::Up => {
                        if pos <= 1 {
                            pos = total;
                        }
                        else {
                            pos -= 1;
                        }
                    },
                    _ => {
                        if pos >= total {
                            pos = 1;
                        }
                        else {
                            pos += 1;
                        }
                    }
                }

                // // Erases current line
                // print!("{}[", '\x1B');
                // print!("1K");

                // Moves cursor to previous 'n' line
                print!("{}[", '\x1B');
                print!("{}F", total); 

                // Moves cursor to beginning line
                print!("{}[", '\x1B');
                print!("1G");

                let mut current_idx = 1;
                for option in &options {
                    if selected.contains(&current_idx) {
                        t.fg(term::color::GREEN).unwrap();
                        print!("{} ", '\u{29BF}');
                        t.reset().unwrap();
                        print!("{}\r\n", option.key);
                    }
                    else if current_idx == pos {
                        t.fg(term::color::GREEN).unwrap();
                        print!("{} ", '\u{29BE}');
                        t.reset().unwrap();
                        print!("{}\r\n", option.key);
                    }
                    else {
                        print!("{} ", '\u{29BE}');
                        print!("{}\r\n", option.key);
                    }

                    current_idx += 1;
                }
            }
        }
    }

    // Show Cursor
    print!("{}[", '\x1B');
    print!("?25h");
    t.reset().unwrap();
    String::from("")
}
