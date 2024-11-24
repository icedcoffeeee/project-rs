use crate::{fs, path};

const OUTPUT_FOLDER: &str = "output";

pub fn get_save_filepath(name: &str) -> String {
    let mut i = 0;
    let mut filepath = path::PathBuf::new();

    filepath.push(OUTPUT_FOLDER);
    if !filepath.exists() {
        fs::create_dir(filepath.clone()).unwrap();
    }

    for item in filepath.read_dir().unwrap() {
        if let Some((num_str, _)) = item
            .unwrap()
            .file_name()
            .into_string()
            .unwrap()
            .split_once("-")
        {
            if let Ok(num) = num_str.parse() {
                if num > i {
                    i = num;
                }
            }
        };
    }

    filepath.push(format!("{}-{}", i + 1, name));
    filepath.to_str().unwrap().to_string()
}
