use clap::{load_yaml, App};
use dialoguer::Confirm;
use import_pics::*;
use std::io;
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

    // DEBUG:
    let source_dir = PathBuf::from(from);
    let target_dir = PathBuf::from(to);
    let files = analyze_source_dir(&source_dir, recursive, &extensions).unwrap();
    let grouped_files = group_files(files, &target_dir);
    match prompt_for_copy(&grouped_files) {
        Ok(true) => (),
        _ => {
            println!("Exiting");
            std::process::exit(0)
        }
    };
    let results = copy_files(grouped_files, &target_dir, None);
    summarize_results(&results);
}

fn prompt_for_copy(file_map: &HashMap<String, Vec<ProcessedFile>>) -> io::Result<bool> {
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

    Confirm::new()
        .with_prompt("Do you want to continue?")
        .default(true)
        .interact()
}

fn summarize_results(results: &Vec<Result<u64, CopyError>>) {
    let mut success = vec![];
    let mut failure = vec![];
    for res in results {
        match res {
            Ok(size) => success.push(size),
            Err(e) => failure.push(e)
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
