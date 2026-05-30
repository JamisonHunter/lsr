use std::{fs::{self, DirEntry, Metadata}, io::Error};
use colored::Colorize;

struct Document {
    name: String,
    size: u64,
    is_dir: bool,
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1000000000 {
        let gigabytes: f64 = bytes as f64 / 1000000000 as f64;
        return format!("{:.1}Gb", gigabytes);
    }
    if bytes >= 1000000 {
        let megabytes: f64 = bytes as f64 / 1000000 as f64;
        return format!("{:.1}Mb", megabytes);
    }
    if bytes >= 1000 {
        let kilobytes: f64 = bytes as f64 / 1000 as f64;
        return format!("{:.1}Kb", kilobytes);
    }
    return format!("{}", bytes);
}

fn get_metadata(path: Result<DirEntry, Error>) -> (Metadata, String) {
    let entry = path.unwrap();
    let path_buf = entry.path();

    let metadata = fs::metadata(&path_buf).unwrap();
    let name: String = path_buf.display().to_string();
    
    return (metadata, name);
}

fn get_dir_size(file_path: String) -> u64 {
    let path_str = format!("./{}", file_path);
    let paths = fs::read_dir(path_str).unwrap();
    let mut total_size: u64 = 0;

    for path in paths {
        let (metadata, name) = get_metadata(path);

        if metadata.is_dir() {
            total_size += get_dir_size(name[2..].to_string());
        } else {
            total_size += metadata.len();
        }

    }    
    return total_size;
}

fn main() {
    let paths = fs::read_dir("./").unwrap();
    let mut documents: Vec<Document> = Vec::new();

    for path in paths {
        let (metadata, name) = get_metadata(path);

        let document = Document {
            name: name[2..].to_string(),
            size: metadata.len(),
            is_dir: metadata.is_dir(),
        };
        
        documents.push(document);
    }

    for document in documents {
        if document.is_dir {
            let file_size = get_dir_size(document.name.clone());
            println!("{}", format!("{} {}", document.name, format_bytes(file_size)).blue());
        } else {
            println!("{} {}", document.name, format_bytes(document.size));
        }
    }
}