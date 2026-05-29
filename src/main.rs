use std::fs;

fn main() {
    let paths = fs::read_dir("./").unwrap();
    let mut files: Vec<String> = Vec::new();

    for path in paths {
        files.push(path.unwrap().path().display().to_string());
    }

    for file in files {
        let name = &file[2..];
        println!("{}", name);
    }
}