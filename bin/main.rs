mod fmt;

use std::array::IntoIter;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::io::{Read, Write};

use anyhow::{Context, Result};
use arboard::{Clipboard, ContentType};
use clap::{App, Arg, ArgGroup, ArgMatches, SubCommand};
use thiserror::Error;

use crate::fmt::{is_a_tty, print_error, Colorizer};

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

pub fn main() {
    human_panic::setup_panic!();
    // env_logger::builder().filter_level(log::LevelFilter::Trace).init();

    #[rustfmt::skip]
    let matches = App::new("cliptools")
        .version(VERSION.unwrap_or("unknown"))
        .subcommand(SubCommand::with_name("paste").about("Prints data from clipboard")
            // TODO add control over final newline
            .arg(Arg::with_name("type")
                .help("Format to fetch the data in, if available. Must be one of `url`, `html`, \
                       `pdf`, `png`, `rtf`, or `text`. For other formats, use --system-type, \
                       or prefix your type with an at sign (@).")
                .long("type")
                .short("t")
                .takes_value(true))
            .arg(Arg::with_name("system-type")
                // TODO add a platform-dependent explanation of the data format
                .help("Format to fetch the data in, if available. The expected format is platform \
                       dependent; for a portable alternative, use --type.")
                .long("system-type")
                .takes_value(true))
            .group(ArgGroup::with_name("format")
                .args(&["type", "system-type"]))
            .arg(Arg::with_name("binary")
                .help("Allow binary output. By default, this is disallowed if the output is a \
                       terminal, and disallowed otherwise.")
                .long("binary")
                .min_values(0)
                .max_values(1)
                .possible_values(&["auto", "always", "never"])))
        .subcommand(SubCommand::with_name("list-types").about("Prints types currently in clipboard")
            .arg(Arg::with_name("system")
                .help("Display native content types, instead of using cliptool aliases")
                .long("system")
                .short("s")))
        .subcommand(SubCommand::with_name("copy").about("Set data in clipboard")
            .arg(Arg::with_name("type")
                .help("Format of the data. Must be one of `url`, `html`, \
                       `pdf`, `png`, `rtf`, or `text`. For other formats, use --system-type, \
                       or prefix your type with an at sign (@).")
                .long("type")
                .short("t")
                .takes_value(true))
            .arg(Arg::with_name("system-type")
                .help("Format of the data. The expected format is platform \
                       dependent; for a portable alternative, use --type.")
                .long("system-type")
                .takes_value(true))
            .arg(Arg::with_name("json")
                .help("Expect a JSON map of data formats to content for each format")
                .long("json")
                .short("j"))
            .group(ArgGroup::with_name("format")
                .args(&["type", "system-type", "json"])))
        .get_matches();

    let mut clipboard = Clipboard::new().expect("unable to open clipboard");

    let (sc, sc_matches) = matches.subcommand();
    let ok = match sc {
        "paste" => paste(&mut clipboard, sc_matches.unwrap()),
        "list-types" => list(&mut clipboard, sc_matches.unwrap().is_present("system")),
        "copy" => copy(&mut clipboard, sc_matches.unwrap()),
        "" => Err(CliptoolsError::ArgumentError("you must specify a subcommand".into()).into()),
        _ => Err(CliptoolsError::ArgumentError(format!("unknown subcommand {}", sc)).into()),
    };

    if let Err(s) = ok {
        let cliptools_error = s.downcast_ref::<CliptoolsError>().expect("unexpected error type");
        let colorizer = Colorizer::default();
        print_error(&s, &colorizer);
        std::process::exit(cliptools_error.exit_code())
    }
}

fn paste(board: &mut Clipboard, matches: &ArgMatches) -> Result<()> {
    let binary_allowed = {
        match matches.value_of("binary") {
            Some("auto") => !is_a_tty(false),
            None | Some("always") => true,
            Some("never") => false,
            other => panic!("unexpected value for binary flag: {:?}", other),
        }
    };

    let ct = if let Some(t) = matches.value_of("type") {
        let converted = string_to_ct(t).ok_or_else(|| {
            CliptoolsError::ArgumentError(format!(
                "unknown type: {}; try using --system-type to specify a system native type",
                t
            ))
        })?;
        Some(converted)
    } else {
        matches.value_of("system-type").map(|t| ContentType::Custom(t.into()))
    };

    if let Some(ct) = ct {
        let val = board
            .get_content_for_type(&ct)
            .map_err(|e| anyhow::Error::msg(e.to_string()).context(CliptoolsError::DataNotFound))?;
        show_binary_content(&val, binary_allowed)?;
    } else {
        let val = board
            .get_text()
            .map_err(|e| anyhow::Error::msg(e.to_string()).context(CliptoolsError::DataNotFound))?;
        print!("{}", &val);
    }
    std::io::stdout().flush().map_err(anyhow::Error::from)
}

fn list(board: &mut Clipboard, system: bool) -> Result<()> {
    let types = board
        .get_content_types()
        .map_err(|e| anyhow::Error::msg(e.to_string()).context(CliptoolsError::DataNotFound))?;
    if system {
        for typ in types {
            println!("{}", typ);
        }
    } else {
        let mut converted = types
            .into_iter()
            .map(|s| board.normalize_content_type(s))
            .map(|ct| show_ct(&ct))
            .collect::<Vec<_>>();
        converted.sort();
        converted.dedup();
        for typ in converted {
            println!("{}", typ);
        }
    }
    Ok(())
}

fn copy(board: &mut Clipboard, matches: &ArgMatches) -> Result<()> {
    let map: HashMap<ContentType, Vec<u8>> = if matches.is_present("json") {
        let json: serde_json::Value = serde_json::from_reader(std::io::stdin())
            .context(CliptoolsError::JsonError("cannot read JSON input".into()))?;
        let map = json.as_object().ok_or_else(|| {
            CliptoolsError::JsonError("expected a JSON object at top level".into())
        })?;
        map.iter()
            .map(|(typ, content)| -> Result<(ContentType, Vec<u8>)> {
                let ct = string_to_ct(typ).ok_or_else(|| {
                    CliptoolsError::ArgumentError(format!("unknown type: {}", typ))
                })?;
                let val = content.as_str().ok_or_else(|| {
                    CliptoolsError::JsonError(format!("expected a string under key {}", typ))
                })?;
                Ok((ct, val.bytes().collect()))
            })
            .collect::<Result<HashMap<_, _>>>()?
    } else {
        let ct = if let Some(t) = matches.value_of("type") {
            string_to_ct(t).ok_or_else(|| {
                CliptoolsError::ArgumentError(format!(
                    "unknown type: {}; try using --system-type to specify a system native type",
                    t
                ))
            })?
        } else if let Some(t) = matches.value_of("system-type") {
            ContentType::Custom(t.into())
        } else {
            ContentType::Text
        };
        let mut data = Vec::new();
        std::io::stdin().read_to_end(&mut data).context(CliptoolsError::InternalError)?;
        IntoIter::new([(ct, data)]).collect()
    };

    board
        .set_content_types(map)
        .map_err(|e| anyhow::Error::msg(e.to_string()).context(CliptoolsError::InternalError))
}

fn string_to_ct(s: &str) -> Option<ContentType> {
    Some(match s.to_ascii_lowercase().as_str() {
        "url" => ContentType::Url,
        "html" => ContentType::Html,
        "pdf" => ContentType::Pdf,
        "png" => ContentType::Png,
        "rtf" => ContentType::Rtf,
        "text" => ContentType::Text,
        _ => {
            if s.starts_with('@') {
                ContentType::Custom(s.chars().skip(1).collect())
            } else {
                return None;
            }
        },
    })
}

fn show_binary_content(val: &[u8], binary_allowed: bool) -> Result<()> {
    if !binary_allowed {
        std::str::from_utf8(val).context(CliptoolsError::Utf8Error)?;
    }
    std::io::stdout().write_all(val).expect("unable to flush stdout");
    Ok(())
}

fn show_ct(ct: &ContentType) -> String {
    match ct {
        ContentType::Text => "text".into(),
        ContentType::Html => "html".into(),
        ContentType::Pdf => "pdf".into(),
        ContentType::Png => "png".into(),
        ContentType::Rtf => "rtf".into(),
        ContentType::Url => "url".into(),
        ContentType::Custom(s) => format!("@{}", s),
    }
}

#[derive(Error, Debug)]
pub enum CliptoolsError {
    #[error("data not found")]
    DataNotFound,
    #[error("usage: {0}")]
    ArgumentError(String),
    #[error("data in clipboard is not valid UTF-8; try using `--binary always`")]
    Utf8Error,
    #[error("invalid JSON input: {0}")]
    JsonError(String),
    #[error("internal error")]
    InternalError,
}

impl CliptoolsError {
    /// Converts an error into the exit code.
    ///  - 1 for missing data or clipboard errors
    ///  - 2 for user errors
    pub fn exit_code(&self) -> i32 {
        match self {
            CliptoolsError::DataNotFound => 1,
            CliptoolsError::InternalError => 1,
            CliptoolsError::ArgumentError(_) => 2,
            CliptoolsError::JsonError(_) => 2,
            CliptoolsError::Utf8Error => 2,
        }
    }
}
