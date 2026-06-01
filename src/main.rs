use std::{env, fs::{self, DirEntry, Metadata}, io::Error, path::PathBuf};
use colored::Colorize;

struct Document {
    name: String,
    size: u64,
    is_dir: bool,
    path: PathBuf,
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        let gigabytes: f64 = bytes as f64 / 1_000_000_000 as f64;
        return format!("{:.1}Gb", gigabytes);
    }
    if bytes >= 1_000_000 {
        let megabytes: f64 = bytes as f64 / 1_000_000 as f64;
        return format!("{:.1}Mb", megabytes);
    }
    if bytes >= 1_000 {
        let kilobytes: f64 = bytes as f64 / 1_000 as f64;
        return format!("{:.1}Kb", kilobytes);
    }
    format!("{}", bytes)
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
    println!("  -h, --help     Show this help message");
    println!();
    println!("With no arguments, lists all items in the current directory with their sizes.");
}

fn main() -> Result<(), std::io::Error> {
    let mut args = env::args().skip(1); // skip the program name

    match args.next() {
        None => {
            // pass
        }
        Some(arg) if arg == "-h" || arg == "--help" => {
            print_help();
            return Ok(());
        }
        Some(arg) if arg == "-v" || arg == "--version" => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
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

                let document = Document {
                    name,
                    size: metadata.len(),
                    is_dir: metadata.is_dir(),
                    path: path_buf,
                };
                documents.push(document);
            }
            Err(_) => continue,
        }
    }

    for document in documents {
        if document.is_dir {
            match get_dir_size(document.path.clone()) {
                Ok(file_size) => {
                    println!("{}", format!("{} ({})", document.name, format_bytes(file_size)).blue());
                }
                Err(_) => continue,
            }
        } else {
            println!("{} ({})", document.name, format_bytes(document.size));
        }
    }

    Ok(())
}