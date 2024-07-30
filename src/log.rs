#![allow(dead_code)]

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Crit,
    None,
}

impl LogLevel {
    pub fn value(&self) -> u8 {
        match self {
            LogLevel::Debug => u8::MIN,
            LogLevel::Info => u8::from(30),
            LogLevel::Warn => u8::from(60),
            LogLevel::Crit => u8::from(90),
            LogLevel::None => u8::MAX,
        }
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
macro_rules! dbug {
    ($current_level:expr, $($msg:expr),*) => {
        if LogLevel::Debug.value() >= $current_level.value() {
            println!($($msg),*);
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($current_level:expr, $($msg:expr),*) => {
        if LogLevel::Warn.value() >= $current_level.value() {
            println!($($msg),*);
        }
    };
}

#[macro_export]
macro_rules! crit {
    ($current_level:expr, $($msg:expr),*) => {
        if LogLevel::Crit.value() >= $current_level.value() {
            println!($($msg),*);
        }
    };
}
