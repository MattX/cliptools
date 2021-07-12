// The MIT License (MIT)
//
// Copyright (c) 2015-2016 Kevin B. Knapp
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// This code is taken from https://github.com/clap-rs/clap/blob/0c7da9f5b32bcd6968a70258a4868d439fbc1fc3/src/fmt.rs

#[cfg(not(target_os = "windows"))]
use ansi_term::ANSIString;

#[cfg(not(target_os = "windows"))]
use ansi_term::Colour::{Green, Red, Yellow};

use std::env;
use std::fmt;

#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ColorWhen {
    Auto,
    Always,
    Never,
}

pub fn is_a_tty(stderr: bool) -> bool {
    let stream = if stderr {
        atty::Stream::Stderr
    } else {
        atty::Stream::Stdout
    };
    atty::is(stream)
}

pub fn is_term_dumb() -> bool {
    env::var("TERM").ok() == Some(String::from("dumb"))
}

pub struct ColorizerOption {
    pub use_stderr: bool,
    pub when: ColorWhen,
}

pub struct Colorizer {
    when: ColorWhen,
}

macro_rules! color {
    ($_self:ident, $c:ident, $m:expr) => {
        match $_self.when {
            ColorWhen::Auto => Format::$c($m),
            ColorWhen::Always => Format::$c($m),
            ColorWhen::Never => Format::None($m),
        }
    };
}

impl Colorizer {
    pub fn new(option: ColorizerOption) -> Colorizer {
        let is_a_tty = is_a_tty(option.use_stderr);
        let is_term_dumb = is_term_dumb();
        Colorizer {
            when: match option.when {
                ColorWhen::Auto if is_a_tty && !is_term_dumb => ColorWhen::Auto,
                ColorWhen::Auto => ColorWhen::Never,
                when => when,
            },
        }
    }

    pub fn good<T>(&self, msg: T) -> Format<T>
    where
        T: fmt::Display + AsRef<str>,
    {
        color!(self, Good, msg)
    }

    pub fn warning<T>(&self, msg: T) -> Format<T>
    where
        T: fmt::Display + AsRef<str>,
    {
        color!(self, Warning, msg)
    }

    pub fn error<T>(&self, msg: T) -> Format<T>
    where
        T: fmt::Display + AsRef<str>,
    {
        color!(self, Error, msg)
    }

    pub fn none<T>(&self, msg: T) -> Format<T>
    where
        T: fmt::Display + AsRef<str>,
    {
        Format::None(msg)
    }
}

impl Default for Colorizer {
    fn default() -> Self {
        Colorizer::new(ColorizerOption {
            use_stderr: true,
            when: ColorWhen::Auto,
        })
    }
}

/// Defines styles for different types of error messages. Defaults to Error=Red, Warning=Yellow,
/// and Good=Green
#[derive(Debug)]
pub enum Format<T> {
    /// Defines the style used for errors, defaults to Red
    Error(T),
    /// Defines the style used for warnings, defaults to Yellow
    Warning(T),
    /// Defines the style used for good values, defaults to Green
    Good(T),
    /// Defines no formatting style
    None(T),
}

#[cfg(not(target_os = "windows"))]
impl<T: AsRef<str>> Format<T> {
    fn format(&self) -> ANSIString {
        match *self {
            Format::Error(ref e) => Red.bold().paint(e.as_ref()),
            Format::Warning(ref e) => Yellow.paint(e.as_ref()),
            Format::Good(ref e) => Green.paint(e.as_ref()),
            Format::None(ref e) => ANSIString::from(e.as_ref()),
        }
    }
}

#[cfg(target_os = "windows")]
#[cfg_attr(feature = "lints", allow(match_same_arms))]
impl<T: fmt::Display> Format<T> {
    fn format(&self) -> &T {
        match *self {
            Format::Error(ref e) => e,
            Format::Warning(ref e) => e,
            Format::Good(ref e) => e,
            Format::None(ref e) => e,
        }
    }
}

#[cfg(not(target_os = "windows"))]
impl<T: AsRef<str>> fmt::Display for Format<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.format())
    }
}

#[cfg(target_os = "windows")]
impl<T: fmt::Display> fmt::Display for Format<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.format())
    }
}

#[cfg(all(test, not(target_os = "windows")))]
mod test {
    use super::Format;
    use ansi_term::ANSIString;
    use ansi_term::Colour::{Green, Red, Yellow};

    #[test]
    fn colored_output() {
        let err = Format::Error("error");
        assert_eq!(
            &*format!("{}", err),
            &*format!("{}", Red.bold().paint("error"))
        );
        let good = Format::Good("good");
        assert_eq!(&*format!("{}", good), &*format!("{}", Green.paint("good")));
        let warn = Format::Warning("warn");
        assert_eq!(&*format!("{}", warn), &*format!("{}", Yellow.paint("warn")));
        let none = Format::None("none");
        assert_eq!(
            &*format!("{}", none),
            &*format!("{}", ANSIString::from("none"))
        );
    }
}
