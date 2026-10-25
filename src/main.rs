mod color;
mod config;
mod format;
mod scan;

use color::Color;
use console::Term;
use std::{env, path::Path, process::ExitCode};

const MAX_BAR_WIDTH: usize = 40;
const MIN_BAR_WIDTH: usize = 5;
const EXIT_INCOMPLETE: u8 = 1;
const EXIT_ERROR: u8 = 2;

fn bar_width(total_docs: usize, term: &Term) -> usize {
    let term_width = match term.size_checked() {
        Some((_, cols)) => cols as usize,
        None => return MAX_BAR_WIDTH,
    };

    let digits = total_docs.max(1).to_string().len();
    let overhead = 15 + 2 * digits;

    term_width
        .saturating_sub(overhead)
        .clamp(MIN_BAR_WIDTH, MAX_BAR_WIDTH)
}

#[allow(clippy::too_many_arguments)]
fn render_progress(
    term: &Term,
    message: &str,
    completed: usize,
    total: usize,
    width: usize,
    fill: &Color,
    empty: &Color,
    colors_enabled: bool,
) {
    let fraction = if total == 0 {
        1.0
    } else {
        completed as f64 / total as f64
    };
    let percent = (fraction * 100.0).round() as u32;
    let filled = ((fraction * width as f64).round() as usize).min(width);
    let remaining = width - filled;

    let bar = format!(
        "{}{}",
        fill.paint(&"█".repeat(filled), colors_enabled),
        empty.paint(&"░".repeat(remaining), colors_enabled)
    );

    let _ = term.clear_line();
    let _ = term.write_str(&format!("{} [{}] {}%", message, bar, percent));
    let _ = term.flush();
}

fn print_help() {
    println!("Directory size viewer");
    println!();
    println!(
        "Usage: {} [OPTION]",
        env::args().next().unwrap_or_else(|| "lsr".to_string())
    );
    println!();
    println!("Options:");
    println!("  -a             Show all files including dot files");
    println!("  -h, --help     Show this help message");
    println!("  -v, --version  Show version information");
    println!();
    println!("lsr scans the current directory and takes no positional arguments.");
    println!("With no options, lists all items sorted by size.");
    println!("Colors are read from ./.lsr.conf (or $LSR_CONFIG).");
    println!("Set NO_COLOR to disable colors.");
    println!();
    println!("Exit status:");
    println!("  0  scan complete");
    println!("  1  scan incomplete (some paths could not be read)");
    println!("  2  usage or fatal error");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Run,
    Help,
    Version,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Options {
    show_dot_files: bool,
    action: Action,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            show_dot_files: false,
            action: Action::Run,
        }
    }
}

fn parse_args<I>(args: I) -> Result<Options, String>
where
    I: IntoIterator<Item = String>,
{
    let mut options = Options::default();
    let mut help = false;
    let mut version = false;

    for arg in args {
        match arg.as_str() {
            "-a" => options.show_dot_files = true,
            "-h" | "--help" => help = true,
            "-v" | "--version" => version = true,
            _ => {
                return Err(format!(
                    "unexpected argument '{}'; try '--help' for usage",
                    format::escape_name(&arg)
                ));
            }
        }
    }

    options.action = if help {
        Action::Help
    } else if version {
        Action::Version
    } else {
        Action::Run
    };

    Ok(options)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScanStatus {
    Complete,
    Incomplete,
}

fn run() -> Result<ScanStatus, String> {
    let options = parse_args(env::args().skip(1))?;

    match options.action {
        Action::Help => {
            print_help();
            return Ok(ScanStatus::Complete);
        }
        Action::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return Ok(ScanStatus::Complete);
        }
        Action::Run => {}
    }

    let config = config::Config::load();
    let stdout_colors = Term::stdout().features().colors_supported();
    let progress_term = Term::stderr();
    let stderr_colors = progress_term.features().colors_supported();

    let mut issues = Vec::new();
    let mut documents = scan::collect_documents(Path::new("."), &mut issues)
        .map_err(|error| format!("cannot read current directory: {error}"))?;

    let total_docs = documents.len();
    let show_progress = progress_term.is_term() && total_docs > 0;
    let progress_width = bar_width(total_docs, &progress_term);

    let mut total_storage: u64 = 0;

    for (index, document) in documents.iter_mut().enumerate() {
        if document.is_dir {
            let result = scan::scan_dir(&document.path, &mut issues);
            document.size = result.bytes;
            document.incomplete = result.incomplete;
        }

        total_storage = total_storage.saturating_add(document.size);

        if show_progress {
            render_progress(
                &progress_term,
                "Scanning",
                index + 1,
                total_docs,
                progress_width,
                &config.progress_fill,
                &config.progress_empty,
                stderr_colors,
            );
        }
    }

    if show_progress {
        let _ = progress_term.clear_line();
        let _ = progress_term.flush();
    }

    for issue in &issues {
        eprintln!(
            "lsr: warning: cannot access {}: {}",
            format::escape_path(&issue.path),
            issue.error
        );
    }

    let incomplete = !issues.is_empty();

    let mut sized_docs: Vec<scan::Document> = documents
        .into_iter()
        .filter(|document| options.show_dot_files || !document.name.starts_with('.'))
        .collect();
    sized_docs.sort_by_key(|document| std::cmp::Reverse(document.size));

    let mut total_output = format!(
        "Current directory size: {}",
        format::format_bytes(total_storage)
    );
    if incomplete {
        total_output.push_str(" (incomplete)");
    }
    println!("{}", config.total.paint(&total_output, stdout_colors));
    if total_storage > 0 {
        println!();
    }

    for document in sized_docs {
        let annotation = if document.incomplete {
            ", incomplete"
        } else {
            ""
        };
        let output = format!(
            "{} ({}{})",
            format::escape_name(&document.name),
            format::format_bytes(document.size),
            annotation
        );

        if document.is_dir {
            println!("{}", config.directory.paint(&output, stdout_colors));
        } else {
            println!("{}", config.file.paint(&output, stdout_colors));
        }
    }

    Ok(if incomplete {
        ScanStatus::Incomplete
    } else {
        ScanStatus::Complete
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(ScanStatus::Complete) => ExitCode::SUCCESS,
        Ok(ScanStatus::Incomplete) => ExitCode::from(EXIT_INCOMPLETE),
        Err(error) => {
            eprintln!("lsr: error: {}", error);
            ExitCode::from(EXIT_ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Options, String> {
        parse_args(args.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn no_arguments_runs_the_scan() {
        assert_eq!(
            parse(&[]).unwrap(),
            Options {
                show_dot_files: false,
                action: Action::Run,
            }
        );
    }

    #[test]
    fn accepts_all_supported_arguments() {
        assert_eq!(
            parse(&["-a"]).unwrap(),
            Options {
                show_dot_files: true,
                action: Action::Run,
            }
        );
        assert_eq!(parse(&["-h"]).unwrap().action, Action::Help);
        assert_eq!(parse(&["--help"]).unwrap().action, Action::Help);
        assert_eq!(parse(&["-v"]).unwrap().action, Action::Version);
        assert_eq!(parse(&["--version"]).unwrap().action, Action::Version);
    }

    #[test]
    fn allows_duplicate_and_combined_valid_flags() {
        assert_eq!(
            parse(&["-a", "-a"]).unwrap(),
            Options {
                show_dot_files: true,
                action: Action::Run,
            }
        );
        assert_eq!(parse(&["-a", "-v"]).unwrap().action, Action::Version);
        assert_eq!(parse(&["-v", "-a"]).unwrap().action, Action::Version);
    }

    #[test]
    fn help_takes_precedence_over_version() {
        assert_eq!(parse(&["-v", "-h"]).unwrap().action, Action::Help);
        assert_eq!(parse(&["-h", "-v"]).unwrap().action, Action::Help);
    }

    #[test]
    fn rejects_unexpected_arguments() {
        let error = parse(&["-a", "unexpected"]).unwrap_err();
        assert!(error.contains("unexpected argument 'unexpected'"));
        assert!(error.contains("--help"));
    }

    #[test]
    fn validates_every_argument_before_handling_help() {
        assert!(parse(&["--help", "unexpected"]).is_err());
    }

    #[test]
    fn escapes_control_characters_in_argument_errors() {
        let error = parse(&["bad\narg"]).unwrap_err();
        assert!(error.contains("bad\\narg"));
        assert!(!error.contains('\n'));
    }
}
