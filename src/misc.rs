/// Collection of useful widgets for doing terminal stuff
use crate::{command_runner::Status, event::Event};
use iced::{
    Background, Element, Length, Theme, font,
    widget::{button, container, text},
};
use iced_core::text::Wrapping;

/// button that sends `Event::Execute` message when pressed. 
/// Passing `Event::Execute`` to `CommandRunner.update()` triggers command execution and output streaming
/// This function is intended to use inside the `view()` function of your application.
pub fn run_button<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    runner_status: &Status,
    on_press: impl Fn(Event) -> Message,
) -> button::Button<'a, Message>
where
    Message: 'a + Clone,
{
    // make button unclickable if it's running
    if Status::is_running(runner_status) {
        button(content)
        .style(
            |theme, _status| 
            button::primary(theme, button::Status::Disabled)
        )
    } else {
        let msg = on_press(Event::Execute);
        button(content)
        .on_press(msg)
        .style(
            |theme, status|
            button::primary(theme, status)
        )
    }
}


/// button that sends `Event::ClearBuffer` message when pressed. 
/// Passing `Event::ClearBuffer`` to `CommandRunner.update()` removes all buffer stored in the `CommandRunner` instance
/// This function is intended to use inside the `view()` function of your application.
pub fn clear_buffer_button<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    runner_status: &Status,
    on_press: impl Fn(Event) -> Message,
) -> button::Button<'a, Message>
where
    Message: 'a + Clone,
{
    // make button unclickable if it's running
    if Status::is_running(runner_status) {
        button(content)
        .style(
            |theme, _status| 
            button::primary(theme, button::Status::Disabled)
            .with_background(Background::Color(iced::Color::TRANSPARENT))
        )
    } else {
        let msg = on_press(Event::ClearBuffer);
        button(content)
        .on_press(msg)
        .style(
            |theme, status|
            button::primary(theme, status)
            .with_background(Background::Color(iced::Color::TRANSPARENT))
        )
    }
}

/// A bar showing current status of command execution
pub fn status_bar<'a, Message>(runner_status: &Status) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let (content, style_fn) = match runner_status {
        Status::Failed(message) => {
            let message = format!("Failed due to the following error: {}", message);
            let message = text(message)
                .font(font::Font {
                    style: font::Style::Italic,
                    ..Default::default()
                })
                .wrapping(Wrapping::Word);

            // need to provide explicit fn type to the compiler
            let style_fn: fn(&Theme) -> container::Style = |theme: &Theme| {
                let palette = theme.palette();
                container::Style {
                    background: Some(Background::Color(palette.background.stronger.color)),
                    text_color: Some(palette.background.stronger.text),
                    ..Default::default()
                }
            };

            (message, style_fn)
        }
        Status::Running => {
            let message = "Running".to_string();
            let message = text(message)
                .font(font::Font {
                    style: font::Style::Italic,
                    ..Default::default()
                })
                .wrapping(Wrapping::Word);

            let style_fn: fn(&Theme) -> container::Style = |theme: &Theme| {
                let palette = theme.palette();
                container::Style {
                    background: Some(Background::Color(palette.background.stronger.color)),
                    text_color: Some(palette.background.stronger.text),
                    ..Default::default()
                }
            };

            (message, style_fn)
        }
        Status::Initialize => {
            let message = "Initializing".to_string();
            let message = text(message).wrapping(Wrapping::Word);

            let style_fn: fn(&Theme) -> container::Style = |theme: &Theme| {
                let palette = theme.palette();
                container::Style {
                    background: Some(Background::Color(palette.background.stronger.color)),
                    text_color: Some(palette.background.stronger.text),
                    ..Default::default()
                }
            };

            (message, style_fn)
        }
        Status::Idle => {
            let message = "Ready".to_string();
            let message = text(message).wrapping(Wrapping::Word);
            let style_fn: fn(&Theme) -> container::Style = |theme: &Theme| {
                let palette = theme.palette();
                container::Style {
                    background: Some(Background::Color(palette.background.strong.color)),
                    text_color: Some(palette.background.strong.text),
                    ..Default::default()
                }
            };

            (message, style_fn)
        }
    };

    container(content)
        .width(Length::Fill)
        .padding(5.0)
        .style(style_fn)
        .into()
}
