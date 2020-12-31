use clap::{App, load_yaml};
use import_pics::analyze_dir;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let _from = matches.value_of("FROM").expect("Source folder must be specified");
    let _to = matches.value_of("TO").expect("Destination folder must be specified");

    // DEBUG:
    println!(
        "{:?}",
        analyze_dir(r"C:\Users\mstancs\Pictures\Screenshots", None).unwrap()
    );
}