// Merge opmode
use clap::{value_parser, Arg, ArgAction, Command};
use std::path::PathBuf;

use super::{SHARED_DATA_ARGS, SHARED_GENERAL_ARGS};

pub fn subcommand() -> Command {
    Command::new("merge")
        .short_flag('m')
        .long_flag("merge")
        .arg_required_else_help(true)
        .about(
            "Merge (A, B) into C and dump results.
A & B must have same format (strictly).
You can two IONex together but it is impossible
to merge Observation RINex into Meteo RINex.",
        )
        .arg(
            Arg::new("file")
                .value_parser(value_parser!(PathBuf))
                .value_name("FILEPATH")
                .action(ArgAction::Set)
                .required(true)
                .help("RINEX file to merge."),
        )
        .next_help_heading("Production Environment")
        .args(SHARED_GENERAL_ARGS.iter())
        .next_help_heading("Data context")
        .args(SHARED_DATA_ARGS.iter())
}
