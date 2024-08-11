#![allow(dead_code)]

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Verbose,
    Info,
    Warning,
    Critical,
    None,
}

impl LogLevel {
    pub fn value(&self) -> u8 {
        match self {
            LogLevel::Debug => u8::MIN,
            LogLevel::Verbose => u8::from(30),
            LogLevel::Info => u8::from(60),
            LogLevel::Warning => u8::from(90),
            LogLevel::Critical => u8::from(120),
            LogLevel::None => u8::MAX,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[macro_export]
macro_rules! info {
    ($current_level:expr, $($msg:expr),*) => {
        if LogLevel::Info.value() >= $current_level.value() {
            println!($($msg),*);
        }
    };
}

#[macro_export]
macro_rules! verb {
    ($current_level:expr, $($msg:expr),*) => {
        if LogLevel::Verbose.value() >= $current_level.value() {
            println!($($msg),*);
        }
    };
}

#[macro_export]
macro_rules! dbug {
    ($current_level:expr, $($msg:expr),*) => {
        if LogLevel::Debug.value() >= $current_level.value() {
            print!("[Debug]");
            println!($($msg),*);
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($current_level:expr, $($msg:expr),*) => {
        if LogLevel::Warning.value() >= $current_level.value() {
            eprintln!($($msg),*);
        }
    };
}

#[macro_export]
macro_rules! crit {
    ($current_level:expr, $($msg:expr),*) => {
        if LogLevel::Critical.value() >= $current_level.value() {
            eprintln!($($msg),*);
        }
    };
}
