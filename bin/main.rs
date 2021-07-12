mod fmt;

use crate::fmt::{is_a_tty, Colorizer, print_error};
use anyhow::{Result, Context};
use clap::{App, Arg, ArgGroup, ArgMatches, SubCommand};
use copypasta::{get_clipboard_context, ClipboardContext, ClipboardProvider, ContentType};
use std::fmt::Formatter;
use std::io::Write;
use thiserror::Error;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

pub fn main() {
    human_panic::setup_panic!();

    let matches = App::new("cliptools")
        .version(VERSION.unwrap_or("unknown"))
        .subcommand(SubCommand::with_name("paste").about("Prints data from clipboard")
            // TODO add control over final newline
            .arg(Arg::with_name("type")
                .help("Format to fetch the data in, if available. Must be one of `url`, `html`, \
                       `pdf`, `png`, `rtf`, or `text`. For other formats, use --custom-type.")
                .long("type")
                .short("t")
                .takes_value(true))
            .arg(Arg::with_name("custom-type")
                .help("Format to fetch the data in, if available. The expected format is platform \
                       dependent; for a portable but less flexible alternative, use --type.")
                .long("custom-type")
                .takes_value(true))
            .group(ArgGroup::with_name("format")
                .args(&["type", "custom-type"]))
            .arg(Arg::with_name("binary")
                .help("Allow binary output. By default, this is disallowed if the output is a \
                       terminal, and disallowed otherwise.")
                .long("binary")
                .min_values(0)
                .max_values(1)
                .possible_values(&["auto", "always", "never"])))
        .subcommand(SubCommand::with_name("list-types").about("Prints types currently in clipboard"))
        .get_matches();

    let clipboard = get_clipboard_context().expect("unable to open clipboard");

    let (sc, sc_matches) = matches.subcommand();
    let ok = match sc {
        "paste" => paste(&clipboard, sc_matches.unwrap()),
        "list-types" => list(&clipboard),
        "" => Err(CliptoolsError::ArgumentError("you must specify a subcommand".into()).into()),
        _ => Err(CliptoolsError::ArgumentError(format!("error: unknown subcommand {}", sc)).into())
    };

    if let Err(s) = ok {
        let cliptools_error = s.downcast_ref::<CliptoolsError>().expect("unexpected error type");
        let colorizer = Colorizer::default();
        print_error(&s, &colorizer);
        std::process::exit(cliptools_error.exit_code())
    }
}

fn paste(board: &ClipboardContext, matches: &ArgMatches) -> Result<()> {
    let binary_allowed = {
        match matches.value_of("binary") {
            Some("auto") => !is_a_tty(false),
            None | Some("always") => true,
            Some("never") => false,
            other => panic!("unexpected value for binary flag: {:?}", other),
        }
    };

    let ct = if let Some(t) = matches.value_of("type") {
        let converted = string_to_ct(t).ok_or(CliptoolsError::ArgumentError(format!(
            "unknown type: {}; try using --custom-type to specify a custom type",
            t
        )))?;
        Some(converted)
    } else {
        matches
            .value_of("custom-type")
            .map(|t| ContentType::Custom(t.into()))
    };

    if let Some(ct) = ct {
        let val = board.get_content_for_type(&ct).expect("unable to read from clipboard");
        show_binary_content(&val, binary_allowed)?;
    } else {
        // TODO change the get_contents() return type to distinguish internal and external errors
        let val = board.get_contents().expect("unable to read from clipboard");
        show_string(&val);
    }
    std::io::stdout().flush().map_err(|e| anyhow::Error::from(e))
}

fn list(board: &ClipboardContext) -> Result<()> {
    let types = board.get_content_types().expect("unable to read content types");
    for typ in types {
        println!("{}", DisplayCt(typ));
    }
    Ok(())
}

fn string_to_ct(s: &str) -> Option<ContentType> {
    Some(match s.to_ascii_lowercase().as_str() {
        "url" => ContentType::Url,
        "html" => ContentType::Html,
        "pdf" => ContentType::Pdf,
        "png" => ContentType::Png,
        "rtf" => ContentType::Rtf,
        "text" => ContentType::Text,
        _ => return None,
    })
}

fn show_binary_content(val: &Option<Vec<u8>>, binary_allowed: bool) -> Result<()> {
    if let Some(v) = val {
        if !binary_allowed {
            std::str::from_utf8(v).context(CliptoolsError::Utf8Error)?;
        }
        std::io::stdout().write_all(v).expect("unable to flush stdout");
    } else {
        return Err(CliptoolsError::DataNotFound.into());
    }
    Ok(())
}

fn show_string(val: &str) {
    print!("{}", val);
}

struct DisplayCt(pub ContentType);

impl std::fmt::Display for DisplayCt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            ContentType::Text => write!(f, "text"),
            ContentType::Html => write!(f, "html"),
            ContentType::Pdf => write!(f, "pdf"),
            ContentType::Png => write!(f, "png"),
            ContentType::Rtf => write!(f, "rtf"),
            ContentType::Url => write!(f, "url"),
            ContentType::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Error, Debug)]
pub enum CliptoolsError {
    #[error("data not found")]
    DataNotFound,
    #[error("argument error: {0}")]
    ArgumentError(String),
    #[error("data in clipboard is binary; try using `--binary always`")]
    Utf8Error,
}

impl CliptoolsError {
    /// Converts an error into the exit code.
    ///  - 1 for missing data
    ///  - 2 for user errors
    pub fn exit_code(&self) -> i32 {
        match self {
            CliptoolsError::DataNotFound => 1,
            CliptoolsError::ArgumentError(_) => 2,
            CliptoolsError::Utf8Error => 2,
        }
    }
}
