#![doc = include_str!("../README.md")]

mod argument;
mod command_runner;
pub mod event;
mod misc;
pub use command_runner::{CommandRunner, Status, Style};
use std::rc::Rc;

pub use argument::{Argument, StreamMode, run_command, run_command_stream_by_buffer, run_command_stream_by_line};
pub use event::{Event, Terminal};
pub use misc::{clear_buffer_button, run_button, status_bar};

use iced::{
    Widget,
    alignment::{Horizontal, Vertical},
    widget::{column, container, row, space},
};

/// Create CommandRunner instance
pub fn create_runner(command: impl Into<String>, args: impl IntoIterator<Item = impl Into<String>>) -> CommandRunner {
    CommandRunner::new(command, args)
}

/// Helper function for creating a templated iced container for everything you need for a functional widget that executes commands
/// Wrap the result in a container to apply padding, alignment, or a style.
///
/// ```rust
/// use iced::{Widget, widget::container};
/// use iced_command_runner::{terminal_container, CommandRunner, Event};
///
/// #[derive(Clone, Debug)]
/// enum Message {
///     Runner(Event),
/// }
///
/// struct App {
///     runner: CommandRunner,
/// }
///
/// impl App {
///     fn view(&self) -> impl Widget<Message> + '_ {
///         container(terminal_container(
///             &self.runner,
///             Message::Runner,
///             "Run",
///             "Clear",
///         ))
///         .padding(5)
///         .style(container::rounded_box)
///     }
/// }
/// ```
pub fn terminal_container<'a, Message, W>(
    runner: &'a CommandRunner,
    on_update: impl Fn(event::Event) -> Message + 'a,
    execute: W,
    clear_buffer: W,
) -> impl Widget<Message> + 'a
where
    Message: Clone + 'a,
    W: iced::Widget<Message> + 'a,
{
    let on_update = Rc::new(on_update);

    let terminal_view_mapper = on_update.clone();
    let execute_button_mapper = on_update.clone();
    let clear_buffer_mapper = on_update.clone();

    let terminal_window =
        runner.crate_view(move |event| (terminal_view_mapper)(event));

    let execute = run_button(
        container(execute.boxed()).align_x(Horizontal::Center),
        &runner.status,
        move |event| (execute_button_mapper)(event),
    );

    let clear_buffer = clear_buffer_button(
        container(clear_buffer.boxed()).align_x(Horizontal::Center),
        &runner.status,
        move |event| (clear_buffer_mapper)(event),
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