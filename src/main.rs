#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate loggerv;
extern crate pathdiff;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate walkdir;

mod repository;
mod repofile;
mod pathclassifier;
mod chunker;

use failure::{
    Context,
    Error,
    ResultExt,
};
use repository::Repository;
use std::path::PathBuf;

#[derive(Debug, Fail)]
enum CliError {
    #[fail(display = "can not get repo_path from matches")] CanNotGetRepoPathFromMatches,
}

fn main() {
    if let Err(e) = run() {
        for cause in e.causes() {
            error!("{}", cause);
        }

        trace!("{}", e.backtrace());

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

    loggerv::init_with_level(value_t!(matches, "log_level", log::LogLevel)?)?;
    trace!("main::run: matches - {:#?}", matches);

    match matches.subcommand_name() {
        Some("add_remote") => run_add_remote(matches.subcommand_matches("add_remote").unwrap())?,
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

fn run_add_remote(_matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_clone(matches: &clap::ArgMatches) -> Result<(), Error> {
    let source_path: PathBuf = matches
        .value_of("source_path")
        .ok_or_else(|| Context::new("can not get source_path from matches"))?
        .into();

    let destination_path: PathBuf = if matches.is_present("destination_path") {
        matches
            .value_of("destination_path")
            .ok_or_else(|| Context::new("can not get destination_path from matches"))?
            .into()
    } else {
        let source_path = source_path.clone();
        let mut components = source_path.components().collect::<Vec<_>>();
        trace!("main::run_clone: components - {:?}", components);

        components.reverse();

        let basename = components
            .get(0)
            .ok_or_else(|| Context::new("can not get basename from source_path"))?
            .as_os_str()
            .into();

        trace!("main::run_clone: basename - {:?}", basename);

        basename
    };

    info!(
        "Cloning from {} to {}",
        source_path.display(),
        destination_path.display()
    );

    let mut repo = Repository::default().with_path(destination_path);

    repo.clone(source_path).context("can not clone repository")?;

    trace!("main::run_init: repo - {:#?}", repo);

    Ok(())
}

fn run_drop(_matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_get(_matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_init(matches: &clap::ArgMatches) -> Result<(), Error> {
    let repo_path: PathBuf = matches
        .value_of("repo_path")
        .ok_or(CliError::CanNotGetRepoPathFromMatches)?
        .into();

    info!("Initializing repository in {}", repo_path.display());

    let mut repo = Repository::default().with_path(repo_path);

    repo.init().context("can not initialize repository")?;

    trace!("main::run_init: repo - {:#?}", repo);

    Ok(())
}

fn run_remote(_matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_sync(_matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_type(_matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}

fn run_watch(_matches: &clap::ArgMatches) -> Result<(), Error> {
    unimplemented!()
}
