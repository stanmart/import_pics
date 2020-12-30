use clap::{App, load_yaml};

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let from = matches.value_of("FROM").expect("Source folder must be specified");
    let to = matches.value_of("TO").expect("Destination folder must be specified");
}