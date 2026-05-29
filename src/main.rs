use std::fs;

fn main() {
    let paths = fs::read_dir("./").unwrap();
    let mut file_names: Vec<String> = Vec::new();

    for path in paths {
        file_names.push(path.unwrap().path().display().to_string());
    }

    for name in file_names {
        let file = fs::File::create(&name).unwrap();
        let file_size = file.metadata().unwrap().len();

        let name = &name[2..];
        println!("{} {}", name, file_size);
    }
}
