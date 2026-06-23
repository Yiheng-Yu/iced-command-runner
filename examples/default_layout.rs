use iced_command_runner::{
    CommandRunner, 
    event::Event,
    create_runner,
    terminal_container,
};

use iced::widget;
use iced::Size;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
    .window_size(Size { width: 500.0, height: 600.0 })
    .run()
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
            runner: create_runner("echo", [""]),
            to_echo: String::new(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let title = widget::text("Default layout");
        let text_prompt = widget::row![
            widget::text("Type something to echo: "),
            widget::text_input("input text here..", &self.to_echo)
            .on_input(Message::Argument)
        ]
        .align_y(iced::alignment::Vertical::Center)
        .width(iced::Length::Fill);

        let terminal_window = terminal_container(
            &self.runner,
            Message::Runner,
            "Run!",
            "Clear history"
        );

        widget::column![title, text_prompt, terminal_window]
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
