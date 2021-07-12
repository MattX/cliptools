mod error;
mod fmt;

use crate::fmt::is_a_tty;
use clap::{App, Arg, ArgGroup, ArgMatches, SubCommand};
use copypasta::{get_clipboard_context, ClipboardContext, ClipboardProvider, ContentType};
use std::fmt::Formatter;
use std::io::Write;

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
        "" => {
            eprintln!("error: you must specify a subcommand");
            std::process::exit(1);
        }
        _ => {
            eprintln!("error: unknown subcommand {}", sc);
            std::process::exit(1);
        }
    };

    if let Err(s) = ok {
        eprintln!("{}", s);
        std::process::exit(1);
    }
}

fn paste(board: &ClipboardContext, matches: &ArgMatches) -> Result<(), String> {
    let binary_allowed = {
        match matches.value_of("binary") {
            None | Some("auto") => is_a_tty(false),
            Some("always") => true,
            Some("never") => false,
            other => panic!("unexpected value for binary flag: {:?}", other),
        }
    };

    let ct = if let Some(t) = matches.value_of("type") {
        let converted = string_to_ct(t).ok_or(format!(
            "unknown type: {}; try using --custom-type to specify a custom type",
            t
        ))?;
        Some(converted)
    } else {
        matches
            .value_of("custom_type")
            .map(|t| ContentType::Custom(t.into()))
    };

    if let Some(ct) = ct {
        let val = board.get_content_for_type(&ct).map_err(|e| e.to_string())?;
        show_binary_content(&val, binary_allowed)?;
    } else {
        let val = board.get_contents().map_err(|e| e.to_string())?;
        show_string(&val);
    }
    std::io::stdout().flush().map_err(|e| e.to_string())
}

fn list(board: &ClipboardContext) -> Result<(), String> {
    let types = board.get_content_types().map_err(|e| e.to_string())?;
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

fn show_binary_content(val: &Option<Vec<u8>>, binary_allowed: bool) -> Result<(), String> {
    if let Some(v) = val {
        let s = std::str::from_utf8(v).map_err(|e| e.to_string())?;
        print!("{}", s);
    } else {
        eprintln!("no data found for this type");
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
