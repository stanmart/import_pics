use clap::{load_yaml, App};
use import_pics::*;
use std::path::PathBuf;

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
    println!("{:?}", grouped_files);
    println!("{:?}", copy_files(grouped_files, &target_dir));
}
