use std::fs;
use colored::Colorize;

struct Document {
    name: String,
    size: u64,
    is_dir: bool,
}

fn main() {
    let paths = fs::read_dir("./").unwrap();
    let mut documents: Vec<Document> = Vec::new();

    for path in paths {
        let entry = path.unwrap();
        let path_buf = entry.path();

        let name: String = path_buf.display().to_string();

        let metadata = fs::metadata(&path_buf).unwrap();

        let document = Document {
            name: name[2..].to_string(),
            size: metadata.len(),
            is_dir: metadata.is_dir(),
        };
        
        documents.push(document);
    }

    for document in documents {
        if document.is_dir {
            println!("{}", format!("{} {} bytes", document.name, document.size).blue());
        } else {
            println!("{} {} bytes", document.name, document.size);
        }
    }
}