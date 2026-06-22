/// Collection of useful widgets for doing terminal stuff
use crate::{command_runner::Status, event::Event};
use iced::{
    Background, Element, Length, Theme, font,
    widget::{button, container, text},
};
use iced_core::text::Wrapping;

/// button that triggers command execution (needs to use with CommandRunner)
pub fn run_button<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    runner_status: &Status,
    on_press: fn(Event) -> Message,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let message = on_press(Event::Execute);
    if runner_status == &Status::Idle {
        button(content).on_press(message).into()
    } else {
        button(content).into()
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
                let palette = theme.extended_palette();
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
                let palette = theme.extended_palette();
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
                let palette = theme.extended_palette();
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
                let palette = theme.extended_palette();
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
