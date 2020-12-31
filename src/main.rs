use std::fs::DirEntry;

use clap::{App, load_yaml};
use import_pics::AnalyzedFile;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let from = matches.value_of("FROM").expect("Source folder must be specified");
    let to = matches.value_of("TO").expect("Destination folder must be specified");

    // DEBUG:
    for entry in std::fs::read_dir(r"C:\Users\mstancs\Pictures\Screenshots").unwrap() {
        let entry = entry.unwrap();
        println!("{:?}", AnalyzedFile::from_direntry(entry));
    }
}