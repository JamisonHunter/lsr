mod color;
mod config;

use color::Color;
use console::Term;
use std::{
    env,
    fs::{self, DirEntry, Metadata},
    io::Error,
    path::PathBuf,
    println,
};

const MAX_BAR_WIDTH: usize = 40;
const MIN_BAR_WIDTH: usize = 5;

struct Document {
    name: String,
    size: u64,
    is_dir: bool,
    path: PathBuf,
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000_000 {
        format!("{:.1} TiB", bytes as f64 / 1_000_000_000_000.0)
    } else if bytes >= 1_000_000_000 {
        format!("{:.1} GiB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.1} MiB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KiB", bytes as f64 / 1_000.0)
    } else {
        format!("{}", bytes)
    }
}

fn bar_width(total_docs: usize) -> usize {
    let term_width = match Term::stdout().size_checked() {
        Some((_, cols)) => cols as usize,
        None => return MAX_BAR_WIDTH,
    };

    let digits = total_docs.max(1).to_string().len();
    let overhead = 15 + 2 * digits;

    term_width
        .saturating_sub(overhead)
        .clamp(MIN_BAR_WIDTH, MAX_BAR_WIDTH)
}

fn render_progress(
    term: &Term,
    message: &str,
    completed: usize,
    total: usize,
    width: usize,
    fill: &Color,
    empty: &Color,
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
        fill.paint(&"█".repeat(filled)),
        empty.paint(&"░".repeat(remaining))
    );

    let _ = term.clear_line();
    let _ = term.write_str(&format!("{} [{}] {}%", message, bar, percent));
    let _ = term.flush();
}

fn get_metadata(path: Result<DirEntry, Error>) -> Result<(Metadata, PathBuf), Error> {
    let entry = path?;
    let path_buf = entry.path();
    let metadata = fs::metadata(&path_buf)?;
    Ok((metadata, path_buf))
}

fn get_dir_size(path: PathBuf) -> Result<u64, std::io::Error> {
    let paths = fs::read_dir(&path)?;
    let mut total_size: u64 = 0;

    for path_entry in paths {
        let entry = path_entry?;
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            total_size += get_dir_size(entry.path())?;
        } else {
            total_size += metadata.len();
        }
    }
    Ok(total_size)
}

fn print_help() {
    println!("Directory size viewer");
    println!();
    println!(
        "Usage: {} [OPTION]",
        env::args().next().unwrap_or_else(|| "dirsize".to_string())
    );
    println!();
    println!("Options:");
    println!("  -a             Show all files including dot files");
    println!("  -h, --help     Show this help message");
    println!("  -v, --version  Show version information");
    println!();
    println!("With no arguments, lists all items sorted by storage size.");
    println!("Colors are read from ./.lsr.conf (or $LSR_CONFIG).");
}

fn main() -> Result<(), std::io::Error> {
    let mut args = env::args().skip(1);

    let mut show_dot_files: bool = false;

    match args.next() {
        None => {}
        Some(arg) if arg == "-h" || arg == "--help" => {
            print_help();
            return Ok(());
        }
        Some(arg) if arg == "-v" || arg == "--version" => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some(arg) if arg == "-a" => {
            show_dot_files = true;
        }
        Some(_) => {
            eprintln!("Invalid argument. Use -h or --help for usage information.");
            std::process::exit(1);
        }
    }

    let config = config::Config::load();

    let paths = fs::read_dir(".")?;
    let mut documents: Vec<Document> = Vec::new();

    for path in paths {
        match get_metadata(path) {
            Ok((metadata, path_buf)) => {
                let name = path_buf
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path_buf.display().to_string());

                documents.push(Document {
                    name,
                    size: metadata.len(),
                    is_dir: metadata.is_dir(),
                    path: path_buf,
                });
            }
            Err(_) => continue,
        }
    }

    let mut sized_docs: Vec<Document> = Vec::new();

    let total_docs = documents.len();
    let progress_term = Term::stderr();
    let show_progress = progress_term.is_term() && total_docs > 0;
    let progress_width = bar_width(total_docs);

    let mut total_storage: u64 = 0;

    for (index, doc) in documents.iter().enumerate() {
        let real_size = if doc.is_dir {
            get_dir_size(doc.path.clone()).unwrap_or(0)
        } else {
            doc.size
        };

        if !show_dot_files {
            if !doc.name.starts_with('.') {
                sized_docs.push(Document {
                    name: doc.name.clone(),
                    size: real_size,
                    is_dir: doc.is_dir,
                    path: doc.path.clone(),
                });
            }
        } else {
            sized_docs.push(Document {
                name: doc.name.clone(),
                size: real_size,
                is_dir: doc.is_dir,
                path: doc.path.clone(),
            });
        }

        total_storage += real_size;

        if show_progress {
            render_progress(
                &progress_term,
                "Scanning",
                index + 1,
                total_docs,
                progress_width,
                &config.progress_fill,
                &config.progress_empty,
            );
        }
    }

    if show_progress {
        let _ = progress_term.clear_line();
        let _ = progress_term.flush();
    }

    sized_docs.sort_by_key(|d| std::cmp::Reverse(d.size));

    let total_storage_output =
        format!("Current directory storage: {}", format_bytes(total_storage));
    println!("{}", config.total.paint(&total_storage_output));
    if total_storage > 0 {
        println!();
    }

    for document in sized_docs {
        let output = format!("{} ({})", document.name, format_bytes(document.size));
        if document.is_dir {
            println!("{}", config.directory.paint(&output));
        } else {
            println!("{}", config.file.paint(&output));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("lsr-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn format_bytes_below_one_kilobyte() {
        assert_eq!(format_bytes(0), "0");
        assert_eq!(format_bytes(999), "999");
    }

    #[test]
    fn format_bytes_unit_boundaries() {
        assert_eq!(format_bytes(1_000), "1.0 KiB");
        assert_eq!(format_bytes(1_000_000), "1.0 MiB");
        assert_eq!(format_bytes(1_000_000_000), "1.0 GiB");
        assert_eq!(format_bytes(1_000_000_000_000), "1.0 TiB");
    }

    #[test]
    fn format_bytes_rounds_to_one_decimal() {
        assert_eq!(format_bytes(1_550), "1.6 KiB");
        assert_eq!(format_bytes(999_999), "1000.0 KiB");
    }

    #[test]
    fn get_dir_size_empty_dir_is_zero() {
        let dir = temp_dir("empty");
        assert_eq!(get_dir_size(dir.clone()).unwrap(), 0);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn get_dir_size_sums_nested_files() {
        let dir = temp_dir("nested");
        fs::write(dir.join("a.txt"), vec![0u8; 100]).unwrap();
        fs::create_dir_all(dir.join("sub/deeper")).unwrap();
        fs::write(dir.join("sub/b.txt"), vec![0u8; 250]).unwrap();
        fs::write(dir.join("sub/deeper/c.txt"), vec![0u8; 50]).unwrap();

        assert_eq!(get_dir_size(dir.clone()).unwrap(), 400);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn get_dir_size_missing_dir_is_error() {
        let dir = env::temp_dir().join("lsr-test-does-not-exist");
        assert!(get_dir_size(dir).is_err());
    }
}
