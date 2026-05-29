use std::fs;
use colored::Colorize;

struct Document {
    name: String,
    size: u64,
    is_dir: bool,
}

fn main() {
    let paths = fs::read_dir("./").unwrap();
    let mut files: Vec<Document> = Vec::new();

    for path in paths {
        let entry = path.unwrap();
        let path_buf = entry.path();

        let name: String = path_buf.display().to_string();

        let metadata = fs::metadata(&path_buf).unwrap();
        let file_size = metadata.len();

        let size: u64 = file_size;

        let document = Document {
            name: name[2..].to_string(),
            size: size,
            is_dir: metadata.is_dir(),
        };
        
        files.push(document);
    }

    for file in files {
        if file.is_dir {
            println!("{}", format!("{} {} bytes", file.name, file.size).blue());
        } else {
            println!("{} {} bytes", file.name, file.size);
        }
    }
}