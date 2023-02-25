use colored::Colorize;
use std::io::Write;
use std::{
    borrow::Cow,
    env,
    ffi::OsStr,
    fmt::{self, Display},
    fs, io,
    path::{Path, PathBuf},
    process,
};

enum Error {
    InCorrectUsage,
    ReadFile(io::Error),
    ParseFile {
        error: syn::Error,
        file: PathBuf,
        source_code: String,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InCorrectUsage => write!(f, "Usage: dump-syntax path/to/filename.rs"),
            Error::ReadFile(error) => write!(f, "Unable to read file: {}", error),
            Error::ParseFile {
                error,
                file,
                source_code,
            } => render_location(f, error, file, source_code),
        }
    }
}

// Render a rustc-style error message, including colors.
//
//     error: Syn unable to parse file
//       --> main.rs:40:17
//        |
//     40 |     fn fmt(&self formatter: &mut fmt::Formatter) -> fmt::Result {
//        |                  ^^^^^^^^^ expected `,`
//
fn render_location(
    formatter: &mut fmt::Formatter,
    err: &syn::Error,
    filepath: &Path,
    code: &str,
) -> fmt::Result {
    let start = err.span().start();
    let mut end = err.span().end();

    if start.line == end.line && start.column == end.column {
        return render_fallback(formatter, err);
    }

    let code_line = match code.lines().nth(start.line - 1) {
        Some(line) => line,
        None => return render_fallback(formatter, err),
    };

    if end.line > start.line {
        end.line = start.line;
        end.column = code_line.len();
    }

    let filename = filepath
        .file_name()
        .map(OsStr::to_string_lossy)
        .unwrap_or(Cow::Borrowed("main.rs"));

    write!(
        formatter,
        "\n\
             {error}{header}\n\
             {indent}{arrow} {filename}:{linenum}:{colnum}\n\
             {indent} {pipe}\n\
             {label} {pipe} {code}\n\
             {indent} {pipe} {offset}{underline} {message}\n\
             ",
        error = "error".red().bold(),
        header = ": Syn unable to parse file".bold(),
        indent = " ".repeat(start.line.to_string().len()),
        arrow = "-->".blue().bold(),
        filename = filename,
        linenum = start.line,
        colnum = start.column,
        pipe = "|".blue().bold(),
        label = start.line.to_string().blue().bold(),
        code = code_line.trim_end(),
        offset = " ".repeat(start.column),
        underline = "^".repeat(end.column - start.column).red().bold(),
        message = err.to_string().red(),
    )
}

fn render_fallback(formatter: &mut fmt::Formatter, err: &syn::Error) -> fmt::Result {
    write!(formatter, "Unable to parse file: {}", err)
}

fn main() {
    if let Err(error) = try_main() {
        _ = writeln!(io::stderr(), "{}", error);
        process::exit(1);
    }
}

fn try_main() -> Result<(), Error> {
    let mut args = env::args_os();
    _ = args.next();

    let filepath = match (args.next(), args.next()) {
        (Some(arg), None) => PathBuf::from(arg),
        _ => return Err(Error::InCorrectUsage),
    };

    let code = fs::read_to_string(&filepath).map_err(Error::ReadFile)?;
    let syntax = syn::parse_file(&code).map_err({
        |error| Error::ParseFile {
            error,
            file: filepath,
            source_code: code,
        }
    })?;

    println!("{:#?}", syntax);
    Ok(())
}
