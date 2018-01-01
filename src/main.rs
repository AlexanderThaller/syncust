#[macro_use]
extern crate log;
extern crate simplelog;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate clap;

use failure::Error;

fn main() {
    if let Err(e) = run() {
        error!("error while running: {}", e);
        ::std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml)
        .name(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .get_matches();

    let _ = simplelog::TermLogger::init(
        value_t!(matches, "log_level", simplelog::LogLevelFilter)?,
        simplelog::Config::default(),
    );
    trace!("matches: {:#?}", matches);

    match matches.subcommand_name() {
        Some("clone") => run_clone(matches.subcommand_matches("clone").unwrap())?,
        Some("drop") => run_drop(matches.subcommand_matches("drop").unwrap())?,
        Some("get") => run_get(matches.subcommand_matches("get").unwrap())?,
        Some("init") => run_init(matches.subcommand_matches("init").unwrap())?,
        Some("remote") => run_remote(matches.subcommand_matches("remote").unwrap())?,
        Some("sync") => run_sync(matches.subcommand_matches("sync").unwrap())?,
        Some("type") => run_type(matches.subcommand_matches("type").unwrap())?,
        Some("watch") => run_watch(matches.subcommand_matches("watch").unwrap())?,
        _ => unreachable!(),
    }

    Ok(())
}

fn run_clone(matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_drop(matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_get(matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_init(matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_remote(matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_sync(matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_type(matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_watch(matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}
