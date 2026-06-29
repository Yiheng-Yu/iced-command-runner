use iced_command_runner::{
    CommandRunner, 
    event::Event,
    terminal_container,
};
use iced::{Element, alignment::Horizontal, Task, Length, widget};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
    .run()
}

#[derive(Clone, Debug)]
enum Message {
    Runner { index: usize, event: Event},
}

struct App {
    runners: Vec<CommandRunner>
}

impl App {
    pub fn new() -> Self {
        Self {
            runners: vec![
                CommandRunner::new("echo", ["'haha echo this first message'", ]).max_lines(25),
                CommandRunner::new("echo", ["'a different message gets echoed'", ]).max_lines(25)
            ]
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let runner1 = terminal_container(
            &self.runners[0],
            |event| Message::Runner {index: 0, event: event},
            "run",
            "clear buffer"
        );

        let runner1 = widget::column![
            widget::text("echo one message"),
            runner1
        ]
        .spacing(10);
        let runner1 = widget::container(runner1)
        .padding(5.0)
        .align_x(Horizontal::Center);

        let runner2 = terminal_container(
            &self.runners[1],
            |event| Message::Runner {index: 1, event: event},
            "run",
            "clear buffer"
        );
        let runner2 = widget::column![
            widget::text("echo something else"),
            runner2
        ]
        .spacing(10);
        let runner2 = widget::container(runner2)
        .padding(5.0)
        .align_x(Horizontal::Center);

        widget::container(
            widget::row![runner1, runner2]
            .spacing(10.0)
            .width(Length::Fill)
        )
        .height(Length::Shrink)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Runner { index: runner_index, event: runner_event } => {
                let to_update = runner_index.clone();
                self.runners[runner_index].update(runner_event).map(
                    move |event| Message::Runner {index: to_update, event: event}
                )
            }
        }
    }
}
