/// Collection of useful widgets for doing terminal stuff
use crate::{command_runner::Status, event::Event};
use iced::{
    Background, 
    Element, 
    Length, 
    Theme, 
    Widget, 
    font, 
    widget::{button, container, text},
};
use iced_core::text::Wrapping;

/// button that sends `Event::Execute` message when pressed.
/// Passing `Event::Execute`` to `CommandRunner.update()` triggers command execution and output streaming
/// This function is intended to use inside the `view()` function of your application.
pub fn run_button<'a, Message, W>(
    content: W,
    runner_status: &Status,
    on_press: impl Fn(Event) -> Message,
) -> button::Button<'a, Message, W>
where
    Message: 'a + Clone,
    W: iced::Widget<Message> + 'a,
{
    button(content)
        .on_press_maybe(
            (!Status::is_running(runner_status))
                .then(|| on_press(Event::Execute))
        )
        .style(button::primary)
}

/// button that sends `Event::ClearBuffer` message when pressed.
/// Passing `Event::ClearBuffer`` to `CommandRunner.update()` removes all buffer stored in the `CommandRunner` instance
/// This function is intended to use inside the `view()` function of your application.
pub fn clear_buffer_button<'a, Message, W>(
    content: W,
    runner_status: &Status,
    on_press: impl Fn(Event) -> Message,
) -> button::Button<'a, Message, W>
where
    Message: 'a + Clone,
    W: iced::Widget<Message> + 'a,
{
    // make button unclickable if it's running
    if Status::is_running(runner_status) {
        button(content).style(|theme, _status| button::primary(theme, button::Status::Disabled))
    } else {
        let msg = on_press(Event::ClearBuffer);
        button(content).on_press(msg).style(button::primary)
    }
}

/// A bar showing current status of command execution
pub fn status_bar<'a, Message>(
    runner_status: &Status,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let (message, italic, strong) = match runner_status {
        Status::Failed(error) => (
            format!("Failed due to the following error: {error}"),
            true,
            false,
        ),
        Status::Running => ("Running".to_string(), true, false),
        Status::Initialize => ("Initializing".to_string(), false, false),
        Status::Idle => ("Ready".to_string(), false, true),
    };

    let mut content = text(message).wrapping(Wrapping::Word);

    if italic {
        content = content.font(font::Font {
            style: font::Style::Italic,
            ..Default::default()
        });
    }

    container(content)
        .width(Length::Fill)
        .padding(5.0)
        .style(move |theme: &Theme| {
            let palette = theme.palette();

            let color = if strong {
                palette.background.strong
            } else {
                palette.background.stronger
            };

            container::Style {
                background: Some(Background::Color(color.color)),
                text_color: Some(color.text),
                ..Default::default()
            }
        })
        .boxed()
}
