use std::fs;

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
            name: name,
            size: size,
            is_dir: false,
        };
        
        files.push(document);
    }

    for file in files {
        let name: &str = &file.name[2..];
        println!("{} {} bytes", name, file.size);
    }
}