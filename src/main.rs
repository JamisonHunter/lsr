use std::fs;

fn main() {
    let paths = fs::read_dir("./").unwrap();
    let mut file_names: Vec<String> = Vec::new();
    let mut file_sizes: Vec<u64> = Vec::new();

    for path in paths {
        let entry = path.unwrap();
        let path_buf = entry.path();

        file_names.push(path_buf.display().to_string());

        let metadata = fs::metadata(&path_buf).unwrap();
        let file_size = metadata.len();

        file_sizes.push(file_size);
    }

    let mut index = 0;
    for name in file_names {
        // let file_size = file.metadata().unwrap().len();

        let name = &name[2..];
        println!("{} {} bytes", name, file_sizes[index]);
        index += 1;
    }
}
