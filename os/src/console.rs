use crate::sbi::console_putchar;
use core::fmt::{self, Write};
use log::{self, Level, LevelFilter, Log, Metadata, Record};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

pub fn init() {
    static LOGGER: MyLogger = MyLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN") => LevelFilter::Warn,
        Some("INFO") => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _ => LevelFilter::Off,
    });
}

pub struct MyLogger;
fn level_to_color_code(level: Level) -> u8 {
    match level {
        Level::Error => 31, // Red
        Level::Warn => 93,  // BrightYellow
        Level::Info => 34,  // Blue
        Level::Debug => 32, // Green
        Level::Trace => 90, // BrightBlack
    }
}
impl Log for MyLogger{
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        println!("\x1b[{}m[{}][{}] {}\x1b[0m",level_to_color_code(record.level()),record.level(),0,record.args());
    }

    fn flush(&self) {}
}

/*#[macro_export]
macro_rules! error {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!(concat!("\x1b[31m[ERROR][0] " ,$fmt), "\x1b[0m\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! warn {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!(concat!("\x1b[93m[WARN][0] " ,$fmt), "\x1b[0m\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!(concat!("\x1b[34m[INFO][0] " ,$fmt), "\x1b[0m\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! debug {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!(concat!("\x1b[32m[DEBUG][0] " ,$fmt), "\x1b[0m\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! trace {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!(concat!("\x1b[90m[TRACE][0] " ,$fmt), "\x1b[0m\n") $(, $($arg)+)?));
    }
}*/