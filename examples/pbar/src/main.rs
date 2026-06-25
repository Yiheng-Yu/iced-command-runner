use iced_command_runner::{
    CommandRunner,
    create_runner,
    terminal_container,
    Event
};
use iced;

fn main() -> iced::Result {
    iced::application(
        new,
        App::update,
        App::view
    )
    .window_size(iced::Size { width: 500.0, height: 600.0 })
    .run()
}

#[derive(Debug, Clone)]
enum Message {
    Runner(Event)
}

struct App {
    pub runner: CommandRunner
}

impl App {
    pub fn view(&self) -> iced::Element<'_, Message> {
        let terminal_window = terminal_container(
                &self.runner,
                Message::Runner,
                "Run dummy progress bar",
                "Clear history"
            );

        let content = iced::widget::column![
            iced::widget::text("Progress bar demo (requires python 'tqdm' package)"),
            terminal_window
        ]
        .spacing(10.0);
        
        iced::widget::container(content)
        .padding(10.0)
        .into()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Runner(event) => {
                self.runner.update(event).map(Message::Runner)
            }
        }
    }
}


fn new() -> App {
    App{ runner: create_runner("python", ["pbar/src/tqdm_test.py"]) }
}