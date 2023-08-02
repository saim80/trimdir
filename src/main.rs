use std::path::Path;
use clap::Parser;
use futures::executor::block_on;

#[derive(Parser, Default)]
struct CLI {
    #[clap(short, long, default_value = "")]
    pattern: String,
    #[clap(short, long, default_value = ".")]
    source_path: std::path::PathBuf,
    #[clap(short, long, default_value = "")]
    target_path: std::path::PathBuf,
}

async fn process_directory(source_path: std::path::PathBuf, target_path: std::path::PathBuf,
                           pattern: String, executor: fn(std::path::PathBuf, std::path::PathBuf)) {
    for entry in std::fs::read_dir(source_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            // check if path's file name matches regex.
            println!("Checking file: {}", path.to_str().unwrap());
            if path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .contains(&pattern)
            {
                executor(path, target_path.clone());
            }
        }
    }
}

async fn iterate_all_directories(paths: Vec<std::path::PathBuf>, pattern: String,
                                 target_path: std::path::PathBuf) {
    let mut futures = Vec::new();
    for path in paths {
        futures.push(
            process_directory(
                path, target_path.clone(),
                pattern.clone(), |path, target_path| {
                    println!("Moving file: {} to {}", path.to_str().unwrap(),
                             target_path.to_str().unwrap());
                    // move the file at path to target_path.
                    let filename = path.file_name().unwrap();
                    let target_path = target_path.join(Path::new(filename));
                    // create subdirectories if they don't exist.
                    if !target_path.parent().unwrap().exists() {
                        std::fs::create_dir_all(target_path.parent().unwrap()).unwrap();
                    }
                    std::fs::rename(path, target_path).unwrap();
                }));
    }
    futures::future::join_all(futures).await;
}

fn main() {
    // Parse command line arguments
    let cli = CLI::parse();
    // make vector for paths.
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    // print received arguments.
    println!("Pattern: {}", cli.pattern);
    println!("Source Path: {}", cli.source_path.to_str().unwrap());
    println!("Target Path: {}", cli.target_path.to_str().unwrap());
    // add the first directory.
    paths.push(cli.source_path);
    // loop through child directories at path and add the directories to the vector.
    for path in paths.clone() {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                paths.push(path);
            }
        }
    }
    // print all paths.
    for path in paths.clone() {
        println!("Path: {}", path.to_str().unwrap());
    }
    block_on(iterate_all_directories(paths, cli.pattern, cli.target_path));
}
