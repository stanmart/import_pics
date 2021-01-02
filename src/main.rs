use clap::{load_yaml, App};
use import_pics::*;
use std::path::PathBuf;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let _from = matches
        .value_of("FROM")
        .expect("Source folder must be specified");
    let _to = matches
        .value_of("TO")
        .expect("Destination folder must be specified");
    let recursive = matches.is_present("recursive");
    let extensions = vec!["jpg", "jpeg", "tif", "tiff", "raw", "arw", "mp4"];

    // DEBUG:
    let path = PathBuf::from(r"C:\Users\mstancs\Pictures\Screenshots");
    let files = list_dir(path, recursive).unwrap();
    let analyzed_files = analyze_files(files, &extensions);
    println!("{:?}", analyzed_files);
}
