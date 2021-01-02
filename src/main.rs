use clap::{load_yaml, App};
use dialoguer::Select;
use import_pics::*;
use std::{collections::HashMap, path::PathBuf};

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let from = matches
        .value_of("FROM")
        .expect("Source folder must be specified");
    let to = matches
        .value_of("TO")
        .expect("Destination folder must be specified");
    let recursive = matches.is_present("recursive");
    let extensions = vec!["jpg", "jpeg", "tif", "tiff", "raw", "arw", "mp4"];

    let source_dir = match validate_source_dir(from) {
        Some(path) => path,
        None => std::process::exit(0),
    };

    let target_dir = match validate_target_dir(to) {
        Some(path) => path,
        None => std::process::exit(0),
    };

    // DEBUG:
    let files = analyze_source_dir(&source_dir, recursive, &extensions).unwrap();
    let grouped_files = group_files(files, &target_dir);
    if !prompt_for_copy(&grouped_files) {
        std::process::exit(0)
    }
    let results = copy_files(grouped_files, &target_dir, None);
    summarize_results(&results);
}

fn prompt_for_copy(file_map: &HashMap<String, Vec<ProcessedFile>>) -> bool {
    fn count_new_existing(files: &Vec<ProcessedFile>) -> (u64, u64) {
        let mut num_new = 0;
        let mut num_old = 0;
        for file in files {
            match file {
                ProcessedFile::New(_) => num_new += 1,
                ProcessedFile::Existing(_) => num_old += 1,
            }
        }
        (num_new, num_old)
    }

    let mut copy_summary: Vec<(String, (u64, u64))> = file_map
        .iter()
        .map(|(subdir, files)| (subdir.clone(), count_new_existing(files)))
        .collect();
    copy_summary.sort_by_key(|item| item.1);

    println!("{:<10} | {:>8} | {:>8}", "", "new", "existing");
    println!("-----------+----------+---------");
    for (subdir, (new, old)) in copy_summary {
        println!("{:<10} | {:>8} | {:>8}", subdir, new, old);
    }
    println!("");

    let select_items = vec!["Copy new files", "Exit"];
    let selection = Select::new().default(0).items(&select_items).interact();
    match selection {
        Ok(0) => true,
        _ => false,
    }
}

fn summarize_results(results: &Vec<Result<u64, CopyError>>) {
    let mut success = vec![];
    let mut failure = vec![];
    for res in results {
        match res {
            Ok(size) => success.push(size),
            Err(e) => failure.push(e),
        }
    }

    println!("Succesfully copied {} files", success.len());
    if failure.len() > 0 {
        println!("The following problems occured:");
        for err in failure {
            println!("{}", err);
        }
    }
}

fn validate_source_dir(from: &str) -> Option<PathBuf> {
    let source_dir = PathBuf::from(from);
    let select_items = vec!["Exit", "Retry"];
    while !source_dir.is_dir() {
        match Select::new()
            .with_prompt("The source directory is not readable / not a directory")
            .items(&select_items)
            .default(0)
            .interact()
        {
            Ok(1) => continue,
            _ => return None,
        }
    }
    Some(source_dir)
}

fn validate_target_dir(to: &str) -> Option<PathBuf> {
    let target_dir = PathBuf::from(to);
    let select_items = vec!["Exit", "Create", "Retry"];
    while !target_dir.is_dir() {
        match Select::new()
            .with_prompt("The target is not an existing directory")
            .items(&select_items)
            .default(0)
            .interact()
        {
            Ok(1) => match std::fs::create_dir_all(&target_dir) {
                Ok(_) => {
                    println!("Succesfully created directory");
                    continue;
                }
                Err(e) => println!("Could not create directory: {}", e),
            },
            Ok(2) => continue,
            _ => return None,
        }
    }
    Some(target_dir)
}
