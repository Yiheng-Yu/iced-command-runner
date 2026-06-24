#![doc = include_str!("../README.md")]

mod argument;
mod command_runner;
pub mod event;
mod misc;
pub use command_runner::{CommandRunner, Status, Style};
use std::rc::Rc;

pub use argument::{Argument, run_command};
pub use event::{Event, Terminal};
pub use misc::{run_button, clear_buffer_button, status_bar};

use iced::{
    Element,
    alignment::{Vertical, Horizontal},
    widget::{
        row,
        column,
        container,
        space
    }
};

/// Create CommandRunner instance
pub fn create_runner(
    command: impl Into<String>, 
    args: impl IntoIterator<Item = impl Into<String>>
) -> CommandRunner {
    CommandRunner::new(command, args)
}

/// Helper function for creating a templated iced container for everything you need for a functional widget that executes commands
/// Example usage:
/// Somewhere in your `view()` function:
/// 
/// ```rust
/// use iced_command_runner::{
///     terminal_container,
///     CommandRunner,
///     Event as RunnerEvent,
/// };
/// 
/// pub enum Message {
///     ..  // your message goes here
///     Runner(RunnerEvent)
/// }
/// 
/// pub struct App {
///     ..// your widgets...
///     runner: CommandRunner,
/// }
/// 
/// impl App {
///     fn view(&self) -> Element<'_, Message> {
///         ...
///         let terminal_window = terminal_container(
///             &runner, 
///             Message::Runner
///             "Run",
///             "Clear",
///         );
///         
///         // terminal_container returns a unformatted iced::widget::container::Container instance,
///         // and you would almost certainly need to implement your own styling function
///         .spacing(5.0)
///         .background(..)
///         .align_x(..)
///         .align_y(..);
///         ...
///     }
/// }
/// ```
pub fn terminal_container<'a, Message>(
    runner: &'a CommandRunner,
    on_update: impl Fn(event::Event) -> Message + 'a,
    execute: impl Into<Element<'a, Message>>,
    clear_buffer: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message>
where
    Message: Clone + 'a
{   
    let on_update = Rc::new(on_update);

    let terminal_view_mapper = on_update.clone();
    let execute_button_mapper = on_update.clone();
    let clear_buffer_mapper = on_update.clone();

    let terminal_window = runner.crate_view(
        move |event| (terminal_view_mapper)(event)
    );

    let execute = run_button(
            container(execute).align_x(Horizontal::Center), 
            &runner.status,
            move |event| (execute_button_mapper)(event)
        );
    
    let clear_buffer = clear_buffer_button(
            container(clear_buffer).align_x(Horizontal::Center),
            &runner.status,
            move |event| (clear_buffer_mapper)(event)
        );

    let buttons = row![
        execute.width(iced::Length::Fill),
        space().width(iced::Length::Fixed(30.0)),
        clear_buffer.width(iced::Length::Fill)
        ]
    .width(iced::Length::Fill)
    .align_y(Vertical::Center);

    let status = status_bar::<Message>(&runner.status);

    let stacked = column![buttons, terminal_window, status]
    .spacing(5.0);
    
    container(stacked)
}