//! Dynamic Argumentation Solved using ASP
use args::{Args, CliTask};
use clap::Parser;
use fallible_iterator::FallibleIterator;
use lib::{
    framework::{
        af::{
            semantics::{self, Program},
            ArgumentationFramework,
        },
        GenericExtension,
    },
    Error, Result,
};

use crate::context::Context;

mod args;
mod context;

pub enum Dynamics {
    No,
    Yes,
}

fn main() -> Result {
    pretty_env_logger::init();

    let args = Args::parse();
    if args.should_show_problems() {
        println!("[CE-CO,CE-ST,SE-CO,SE-ST]");
        Ok(())
    } else if args.should_show_formats() {
        println!("[apx,tgf]");
        Ok(())
    } else if let Some(task) = args.task() {
        // Decide task to execute
        match task {
            CliTask::CeCoD => {
                run_task_count_extensions::<semantics::Complete>(&args, Dynamics::Yes)
            }
            CliTask::CeCo => run_task_count_extensions::<semantics::Complete>(&args, Dynamics::No),
            CliTask::CeSt => run_task_count_extensions::<semantics::Stable>(&args, Dynamics::No),
            CliTask::EeCoD => {
                run_task_enumerate_extensions::<semantics::Complete>(&args, Dynamics::Yes)
            }
            CliTask::EeCo => {
                run_task_enumerate_extensions::<semantics::Complete>(&args, Dynamics::No)
            }
            CliTask::SeCoD => {
                run_task_sample_extension::<semantics::Complete>(&args, Dynamics::Yes)
            }
            CliTask::SeCo => run_task_sample_extension::<semantics::Complete>(&args, Dynamics::No),
            CliTask::SeSt => run_task_sample_extension::<semantics::Stable>(&args, Dynamics::No),
            CliTask::EeStD => {
                run_task_enumerate_extensions::<semantics::Stable>(&args, Dynamics::Yes)
            }
            CliTask::EeSt => {
                run_task_enumerate_extensions::<semantics::Stable>(&args, Dynamics::No)
            }
            CliTask::CeAd => {
                run_task_count_extensions::<semantics::Admissible>(&args, Dynamics::No)
            }
            CliTask::SeAd => {
                run_task_sample_extension::<semantics::Admissible>(&args, Dynamics::No)
            }
            CliTask::EeAd => {
                run_task_enumerate_extensions::<semantics::Admissible>(&args, Dynamics::No)
            }
            CliTask::CeAdD => {
                run_task_count_extensions::<semantics::Admissible>(&args, Dynamics::Yes)
            }
            CliTask::CeStD => run_task_count_extensions::<semantics::Stable>(&args, Dynamics::Yes),
            CliTask::EeAdD => {
                run_task_enumerate_extensions::<semantics::Admissible>(&args, Dynamics::Yes)
            }
            CliTask::SeAdD => {
                run_task_sample_extension::<semantics::Admissible>(&args, Dynamics::Yes)
            }
            CliTask::SeStD => run_task_sample_extension::<semantics::Stable>(&args, Dynamics::Yes),
        }
    } else {
        println!("dasp v0.1");
        println!("Malte Tammena, malte.tammena@pm.me");
        Ok(())
    }
}

fn run_task_enumerate_extensions<P: Program>(args: &Args, dynamics: Dynamics) -> Result {
    let mut ctx = Context::<ArgumentationFramework<P>>::new(args)?;
    ctx.enumerate_extensions()?.by_ref().for_each(|ext| {
        println!("{}", ext.format());
        Ok(())
    })?;
    if matches!(dynamics, Dynamics::Yes) {
        let mut update_iter = args.update_file().lines()?;
        while let Some(update) = update_iter.next()? {
            ctx.update(&update)?;
            log::trace!("Found update: {:?}", update);
            ctx.enumerate_extensions()?.by_ref().for_each(|ext| {
                println!("{}", ext.format());
                Ok(())
            })?;
        }
    }
    Ok(())
}

fn run_task_count_extensions<P: Program>(args: &Args, dynamics: Dynamics) -> Result {
    let mut ctx = Context::<ArgumentationFramework<P>>::new(args)?;
    println!("{}", ctx.count_extensions()?);
    if matches!(dynamics, Dynamics::Yes) {
        let mut update_iter = args.update_file().lines()?;
        while let Some(update) = update_iter.next()? {
            ctx.update(&update)?;
            println!("{}", ctx.count_extensions()?);
        }
    }
    Ok(())
}

fn run_task_sample_extension<P: Program>(args: &Args, dynamics: Dynamics) -> Result {
    let mut ctx = Context::<ArgumentationFramework<P>>::new(args)?;
    match ctx.sample_extension()? {
        Some(ext) => println!("{}", ext.format()),
        None => println!("NO"),
    }
    if matches!(dynamics, Dynamics::Yes) {
        let mut update_iter = args.update_file().lines()?;
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
    use assert_cmd::cargo::CommandCargoExt;
    use pretty_assertions::assert_eq;
    use std::{collections::HashSet, io::Write, process::Output};

    fn assert_output(output: Output, expect: &str) {
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("Output is UTF8");
        assert_eq!(stdout, expect);
    }

    fn parse_extension(line: &str) -> HashSet<String> {
        let cleaned = line
            .trim_matches('\n')
            .trim_start_matches('[')
            .trim_end_matches(']');
        if cleaned.is_empty() {
            HashSet::new()
        } else {
            cleaned.split(',').map(String::from).collect()
        }
    }

    macro_rules! assemble_command {
        ($($content:literal),+; $task:literal) => {
            {
                let mut file = tempfile::NamedTempFile::new().expect("Creating tempfile");
                write!(file, "{}", &[$( $content ),+].join("\n"))
                    .expect("Writing file");
                let mut cmd = ::std::process::Command::cargo_bin("cli")
                    .expect("Cargo binary found");
                cmd.args(&[
                    // Load file
                    "--file",
                    file.path().to_str().unwrap(),
                    // TGF format
                    "--fo",
                    "tgf",
                    // Execute task
                    "--task",
                    $task,
                ]);
                cmd.stderr(::std::process::Stdio::null());
                (rexpect::session::spawn_command(cmd, Some(2000)).expect("Spawning command"), file)
            }
        };
    }

    macro_rules! expect_extensions {
        ($session:expr, $( [ $( $atom:literal ),* ] ),* ) => {{
            let session: &mut rexpect::session::PtySession = $session;
            let expected_extensions: &[&[&str]] = &[ $( &[ $( $atom ),* ] ),* ];
            let mut expected_extensions: Vec<HashSet<String>> = expected_extensions
                .iter()
                .map(|ext| ext.iter().cloned().map(String::from).collect())
                .collect();
            while !expected_extensions.is_empty() {
                let line = session.read_line().expect("Expecting another extension, but failed to read output");
                let actual_extension = parse_extension(&line);
                let maybe_ext_idx = expected_extensions.iter().position(|ext| *ext == actual_extension);
                match maybe_ext_idx {
                    None => panic!("Extension {line:?} not found in list of expected extensions {expected_extensions:?}"),
                    Some(idx) => { expected_extensions.remove(idx); },
                }
            }
        }}
    }

    fn parse_assert_extensions(output: Output, expect: &[&[&str]]) {
        let extensions = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(parse_extension)
            .filter(|ext| !ext.is_empty())
            .collect::<Vec<_>>();
        assert_eq!(
            extensions.len(),
            expect.len(),
            "{:?} <-> {:?}",
            extensions,
            expect
        );
        for expected_ext in expect.iter().map(|ext| {
            ext.iter()
                .cloned()
                .map(String::from)
                .collect::<HashSet<_>>()
        }) {
            assert!(
                extensions.contains(&expected_ext),
                "Missing extension from {:?}: {:?}",
                extensions,
                expected_ext
            );
        }
    }

    #[test]
    fn enumerate_complete_extensions() {
        let (mut session, _file) = assemble_command! {
            "1", "2", "3", "#", "1 2", "1 1";
            "ee-co"
        };
        expect_extensions!(&mut session, ["3"]);

        let (mut session, _file) = assemble_command! {
            "1", "2", "3", "#", "1 2", "2 1";
            "ee-co"
        };
        expect_extensions!(&mut session, ["3"], ["1", "3"], ["2", "3"]);

        let (mut session, _file) = assemble_command! {
            "1", "2", "3", "#", "1 2", "2 1";
            "ee-co-d"
        };
        expect_extensions!(&mut session, ["3"], ["1", "3"], ["2", "3"]);
        session.send_line("-3").expect("Sending update");
        expect_extensions!(&mut session, ["1"], ["2"], []);
        session.send_line("+3").expect("Sending update");
        expect_extensions!(&mut session, ["3"], ["1", "3"], ["2", "3"]);
        session.send_line("+3").expect("Sending update");
        expect_extensions!(&mut session, ["3"], ["1", "3"], ["2", "3"]);
    }

    #[test]
    fn count_complete_extensions() {
        let mut file = tempfile::NamedTempFile::new().expect("Creating tempfile");
        write!(file, "{}", &["1", "2", "3", "#", "1 2", "1 1"].join("\n")).expect("Writing file");
        let output = assert_cmd::Command::cargo_bin("cli")
            .expect("Cargo binary found")
            .args(&[
                // Load file
                "--file",
                file.path().to_str().unwrap(),
                // TGF format
                "--fo",
                "tgf",
                // Execute task
                "--task",
                "ce-co",
            ])
            .unwrap();
        assert_output(output, "1\n");

        let mut file = tempfile::NamedTempFile::new().expect("Creating tempfile");
        write!(file, "{}", &["1", "2", "3", "#", "1 2", "2 1"].join("\n")).expect("Writing file");
        let output = assert_cmd::Command::cargo_bin("cli")
            .expect("Cargo binary found")
            .args(&[
                // Load file
                "--file",
                file.path().to_str().unwrap(),
                // TGF format
                "--fo",
                "tgf",
                // Execute task
                "--task",
                "ce-co",
            ])
            .unwrap();
        assert_output(output, "3\n");
    }

    #[test]
    fn enumerate_stable_extensions() {
        let mut file = tempfile::NamedTempFile::new().expect("Creating tempfile");
        write!(file, "{}", &["1", "2", "3", "#", "1 2", "2 1"].join("\n")).expect("Writing file");
        let output = assert_cmd::Command::cargo_bin("cli")
            .expect("Cargo binary found")
            .args(&[
                // Load file
                "--file",
                file.path().to_str().unwrap(),
                // TGF format
                "--fo",
                "tgf",
                // Execute task
                "--task",
                "ee-st",
            ])
            .unwrap();
        parse_assert_extensions(output, &[&["1", "3"], &["2", "3"]]);

        let mut file = tempfile::NamedTempFile::new().expect("Creating tempfile");
        write!(file, "{}", &["1", "2", "3", "#", "1 2"].join("\n")).expect("Writing file");
        let output = assert_cmd::Command::cargo_bin("cli")
            .expect("Cargo binary found")
            .args(&[
                // Load file
                "--file",
                file.path().to_str().unwrap(),
                // TGF format
                "--fo",
                "tgf",
                // Execute task
                "--task",
                "ee-st",
            ])
            .unwrap();
        parse_assert_extensions(output, &[&["1", "3"]]);
    }
}
