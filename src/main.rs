use rayon::prelude::*;
use std::env;
use std::process::Command;
use walkdir::WalkDir;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide the target directory as a command-line argument");
        return;
    }
    let target_directory = &args[1];

    let current_dir = env::current_dir().expect("Failed to get current directory");

    let target_dirs: Vec<_> = WalkDir::new(target_directory)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if entry.file_type().is_file() {
                if path.file_name() == Some("pubspec.yaml".as_ref()) {
                    Some(path.parent().unwrap().to_path_buf())
                } else if path.file_name() == Some("Cargo.toml".as_ref()) {
                    Some(path.parent().unwrap().to_path_buf())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    target_dirs.par_iter().for_each(|dir_path| {
        if dir_path != &current_dir {
            println!("Processing: {:?}", dir_path);

            if dir_path.join("pubspec.yaml").exists() {
                Command::new("flutter")
                    .arg("clean")
                    .arg("&&")
                    .arg("flutter")
                    .arg("pub")
                    .arg("get")
                    .current_dir(dir_path)
                    .output()
                    .expect("Failed to execute Flutter commands");
            }

            if dir_path.join("Cargo.toml").exists() {
                Command::new("cargo")
                    .arg("clean")
                    .current_dir(dir_path)
                    .output()
                    .expect("Failed to execute Cargo clean command");
            }

            println!("Done: {:?}", dir_path);
        }
    });
}
