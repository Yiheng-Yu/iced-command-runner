#![doc = include_str!("../README.md")]

mod argument;
mod command_runner;
pub mod event;
mod misc;
pub use command_runner::{CommandRunner, Status, Style};

pub use argument::{Argument, run_command};

pub use misc::{run_button, status_bar};

/// Create CommandRunner instance
pub fn crate_runner(
    command: impl Into<String>, 
    args: impl IntoIterator<Item = impl Into<String>>
) -> CommandRunner {
    CommandRunner::new(command, args)
}
