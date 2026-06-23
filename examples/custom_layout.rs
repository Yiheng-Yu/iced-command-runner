use iced_command_runner::{
    CommandRunner, 
    event::Event,
    run_button, 
    create_runner,
    status_bar
};

use iced::{Length, widget};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
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
        let title = widget::text("Terminal window with no 'clear buffer' button");
        let button = run_button("Run!", &self.runner.status, Message::Runner);

        let text_prompt = widget::row![
            widget::text("Type something to echo: "),
            widget::text_input("input text here..", &self.to_echo)
            .on_input(Message::Argument)
        ]
        .align_y(iced::alignment::Vertical::Center)
        .width(iced::Length::Fill);

        let terminal_window1 = widget::container(
            self.runner.crate_view(Message::Runner)
        )
            .width(iced::Length::Fill);

        let terminal_window2 = widget::container(
            self.runner.crate_view(Message::Runner)
        )
            .width(iced::Length::Fill);

        let terminal_window3 = widget::container(
            self.runner.crate_view(Message::Runner)
        )
            .width(iced::Length::Fill);

        let status_bar = status_bar(&self.runner.status);

        let desc = widget::text("And multile views copies of the same command process:");
        let terminal_window_row = widget::row![terminal_window2, terminal_window3];

        widget::column![title, text_prompt, button, terminal_window1, desc, terminal_window_row, status_bar]
            .spacing(2.5)
            .padding(5.0)
            .height(Length::Shrink)
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
