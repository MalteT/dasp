//! Main CLI for DASP
mod args;
mod path_or_stdin;

use std::time::Instant;

use args::ARGS;
use fallible_iterator::FallibleIterator;
use humantime::format_duration;
use lib::{
    argumentation_framework::{semantics::ArgumentationFrameworkSemantic, ArgumentationFramework},
    semantics, Framework, GenericExtension,
};

use crate::args::CliTask;

pub type Result<T = (), E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Internal error: {_0}")]
    Lib(#[from] lib::Error),
    #[error("IO: {_0}")]
    Io(#[from] std::io::Error),
}

pub enum Dynamics {
    No,
    Yes,
}

fn main() -> Result {
    pretty_env_logger::init();

    log::trace!("Parsed arguments: {:#?}", *ARGS);

    let before = Instant::now();
    let res = match ARGS.task {
        CliTask::CeAd => run_task_count_extensions::<semantics::Admissible>(Dynamics::No),
        CliTask::EeAd => run_task_enumerate_extensions::<semantics::Admissible>(Dynamics::No),
        CliTask::SeAd => run_task_sample_extension::<semantics::Admissible>(Dynamics::No),
        CliTask::CeAdD => run_task_count_extensions::<semantics::Admissible>(Dynamics::Yes),
        CliTask::EeAdD => run_task_enumerate_extensions::<semantics::Admissible>(Dynamics::Yes),
        CliTask::SeAdD => run_task_sample_extension::<semantics::Admissible>(Dynamics::Yes),
    };
    log::info!("Entire solving took {}", format_duration(before.elapsed()));
    res
}

fn load_initial_file_into_af<S: ArgumentationFrameworkSemantic>(
) -> Result<ArgumentationFramework<S>> {
    let content = std::fs::read_to_string(&ARGS.file)?;
    let af = ArgumentationFramework::new(&content)?;
    log::info!("Successfully populated AF from initial file");
    Ok(af)
}

fn run_task_count_extensions<S: ArgumentationFrameworkSemantic>(dynamics: Dynamics) -> Result {
    let mut af = load_initial_file_into_af::<S>()?;
    println!("// Initial count");
    println!("{}", af.count_extensions()?);
    if matches!(dynamics, Dynamics::Yes) {
        let mut update_iter = ARGS.update_file.lines()?.enumerate();
        while let Some((nr, update)) = update_iter.next()? {
            af.update(&update)?;
            println!("// Update #{nr} -- {update}");
            println!("{}", af.count_extensions()?);
        }
    }
    Ok(())
}

fn run_task_enumerate_extensions<S: ArgumentationFrameworkSemantic>(dynamics: Dynamics) -> Result {
    let mut af = load_initial_file_into_af::<S>()?;
    println!("// Initial extensions");
    af.enumerate_extensions()?.by_ref().for_each(|ext| {
        println!("{}", ext.format());
        Ok(())
    })?;
    if matches!(dynamics, Dynamics::Yes) {
        let mut update_iter = ARGS.update_file.lines()?.enumerate();
        while let Some((nr, update)) = update_iter.next()? {
            af.update(&update)?;
            println!("// Update #{nr} -- {update}");
            af.enumerate_extensions()?.by_ref().for_each(|ext| {
                println!("{}", ext.format());
                Ok(())
            })?;
        }
    }
    Ok(())
}

fn run_task_sample_extension<P: ArgumentationFrameworkSemantic>(dynamics: Dynamics) -> Result {
    let mut ctx = load_initial_file_into_af::<P>()?;
    match ctx.sample_extension()? {
        Some(ext) => println!("{}", ext.format()),
        None => println!("NO"),
    }
    if matches!(dynamics, Dynamics::Yes) {
        let mut update_iter = ARGS.update_file.lines()?;
        while let Some(update) = update_iter.next()? {
            ctx.update(&update)?;
            match ctx.sample_extension()? {
                Some(ext) => println!("{}", ext.format()),
                None => println!("NO"),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[test]
    fn argument_parser_works() {
        assert_eq!(
            crate::args::Args::try_parse_from([""]).unwrap_err().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }
}
