use std::{env, fs::{self, DirEntry, Metadata}, io::Error, path::PathBuf, println};
use colored::Colorize;

struct Document {
    name: String,
    size: u64,
    is_dir: bool,
    path: PathBuf,
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        return format!("{:.1}Gb", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        return format!("{:.1}Mb", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        return format!("{:.1}Kb", bytes as f64 / 1_000.0)
    } else {
        return format!("{}", bytes)
    }
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
    println!("Usage: {} [OPTION]", env::args().next().unwrap_or_else(|| "dirsize".to_string()));
    println!();
    println!("Options:");
    println!("  -a             Show all files including hidden files (dot files)");
    println!("  -h, --help     Show this help message");
    println!("  -v, --version  Show version information");
    println!();
    println!("With no arguments, lists all items sorted by size (largest first).");
}

fn main() -> Result<(), std::io::Error> {
    let mut args = env::args().skip(1);

    let mut show_dot_files: bool = false;

    match args.next() {
        None => {},
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

    let paths = fs::read_dir(".")?;
    let mut documents: Vec<Document> = Vec::new();

    for path in paths {
        match get_metadata(path) {
            Ok((metadata, path_buf)) => {
                let name = path_buf.file_name()
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

    let mut total_storage: u64 = 0;

    for doc in documents {
        let real_size = if doc.is_dir {
            get_dir_size(doc.path.clone()).unwrap_or(0)
        } else {
            doc.size
        };

        if show_dot_files == false {
            if doc.name.chars().nth(0).unwrap() != '.' {
                sized_docs.push(Document {
                    name: doc.name,
                    size: real_size,
                    is_dir: doc.is_dir,
                    path: doc.path,
                });
            }
        } else {
            sized_docs.push(Document {
                name: doc.name,
                size: real_size,
                is_dir: doc.is_dir,
                path: doc.path,
            });
        }

        

        total_storage += real_size;
    }

    sized_docs.sort_by_key(|d| std::cmp::Reverse(d.size));

    if total_storage > 0 {
        let total_storage_output = format!("Current directory storage: {}", format_bytes(total_storage));
        println!("{}", total_storage_output.green());
        println!("");
    }

    for document in sized_docs {
        let output = format!("{} ({})", document.name, format_bytes(document.size));
        if document.is_dir {
            println!("{}", output.blue());
        } else {
            println!("{}", output);
        }
    }

    Ok(())
}