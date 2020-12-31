use std::path::PathBuf;
use clap::{App, load_yaml};
use import_pics::analyze_dir;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let _from = matches.value_of("FROM").expect("Source folder must be specified");
    let _to = matches.value_of("TO").expect("Destination folder must be specified");
    let recursive =  matches.is_present("recursive");

    // DEBUG:
    let path = PathBuf::from(r"C:\Users\mstancs\Pictures\Screenshots");
    println!(
        "{:?}",
        analyze_dir(path, None, recursive).unwrap()
    );
}