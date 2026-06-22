use iced_command_runner::{CommandRunner, event::Event, run_button, crate_runner, status_bar};

use iced::widget;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view).run()
}

#[derive(Clone, Debug)]
enum Message {
    Runner(Event),
    Argument(String),
}

struct App {
    runner: CommandRunner,
    to_echo: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            runner: crate_runner("echo", [""]),
            to_echo: String::new(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let button = run_button("Run!", &self.runner.status, Message::Runner);

        let text_prompt = widget::row![
            widget::text("Type something to echo: "),
            widget::text_input("input text here..", &self.to_echo)
            .on_input(Message::Argument)
        ]
        .align_y(iced::alignment::Vertical::Center)
        .width(iced::Length::Fill);

        let terminal_window = widget::container(
            self.runner.crate_view(Message::Runner)
        )
            .width(iced::Length::Fill);

        let status_bar = status_bar(&self.runner.status);

        widget::column![text_prompt, button, terminal_window, status_bar]
            .spacing(2.5)
            .padding(5.0)
            .into()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Runner(event) => {
                self.runner.update(event).map(Message::Runner)
            },
            
            Message::Argument(data) => {
                self.to_echo = data.clone();
                self.runner.set_args([data.clone()]);
                iced::Task::none()
            }
        }
    }
}
