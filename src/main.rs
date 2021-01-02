use clap::{load_yaml, App};
use dialoguer::Select;
use import_pics::*;
use regex::Regex;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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
    let copy_wo_prompt = matches.is_present("yes");
    let extensions = match matches.values_of("extensions") {
        Some(ext) => ext.collect(),
        None => vec!["jpg", "jpeg", "tif", "tiff", "raw", "arw", "mp4"],
    };
    if extensions.len() == 0 {
        println!("At least one extension must be supplied");
        std::process::exit(1);
    }
    let filter_re = match matches.value_of("filter").map(Regex::new) {
        None => None,
        Some(Ok(re)) => Some(re),
        Some(Err(e)) => {
            println!("Could not parse regular expression: {}", e);
            std::process::exit(1);
        }
    };
    let skip_re = match matches.value_of("skip").map(Regex::new) {
        None => None,
        Some(Ok(re)) => Some(re),
        Some(Err(e)) => {
            println!("Could not parse regular expression: {}", e);
            std::process::exit(1);
        }
    };

    let source_dir = match validate_source_dir(from) {
        Some(path) => path,
        None => std::process::exit(0),
    };

    let target_dir = match validate_target_dir(to) {
        Some(path) => path,
        None => std::process::exit(0),
    };

    let files =
        match try_analyze_source_dir(&source_dir, recursive, &extensions, &filter_re, &skip_re) {
            Some(f) => f,
            None => std::process::exit(0),
        };
    let grouped_files = group_files(files, &target_dir);
    let results = match summarize_copy_plan(&grouped_files, copy_wo_prompt) {
        None => std::process::exit(0),
        Some(0) => {
            println!("No new files to copy");
            std::process::exit(0);
        }
        Some(num_files) => copy_files(grouped_files, &target_dir, Some(num_files)),
    };
    summarize_results(&results);
}

fn summarize_copy_plan(
    file_map: &HashMap<String, Vec<ProcessedFile>>,
    copy_wo_prompt: bool,
) -> Option<u64> {
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

    let num_of_files = copy_summary.iter().map(|(_, (n, _))| n).sum();

    println!("{:<10} | {:>8} | {:>8}", "", "new", "existing");
    println!("-----------+----------+---------");
    for (subdir, (new, old)) in copy_summary {
        println!("{:<10} | {:>8} | {:>8}", subdir, new, old);
    }
    println!("");

    if num_of_files == 0 {
        return Some(0);
    }

    if copy_wo_prompt {
        Some(num_of_files)
    } else {
        let select_items = vec!["Copy new files", "Exit"];
        let selection = Select::new().default(0).items(&select_items).interact();
        match selection {
            Ok(0) => Some(num_of_files),
            _ => None,
        }
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

    println!("Succesfully copied {} file(s)", success.len());
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

fn try_analyze_source_dir(
    dir: &Path,
    recursive: bool,
    extensions: &Vec<&str>,
    filter_re: &Option<Regex>,
    skip_re: &Option<Regex>,
) -> Option<Vec<FileWithMetadata>> {
    let select_items = vec!["Exit", "Retry"];
    loop {
        match analyze_source_dir(dir, recursive, &extensions, filter_re, skip_re) {
            Ok(files) => return Some(files),
            Err(e) => {
                println!("Could not analyze source directory: {}", e);
                match Select::new().items(&select_items).default(0).interact() {
                    Ok(1) => continue,
                    _ => return None,
                }
            }
        }
    }
}
