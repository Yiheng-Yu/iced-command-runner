# iced_command_runner

`iced_command_runner` executes commands in child processes, streams their output,
and displays it in an iced terminal widget.

- [Crates.io](https://crates.io/crates/iced_command_runner)
- [Released documentation](https://docs.rs/iced_command_runner/latest/iced_command_runner/)
- [Issues](https://github.com/Yiheng-Yu/iced-command-runner/issues)

## Development dependencies

The `dev` branch follows [iced on GitHub](https://github.com/iced-rs/iced).
Use the same iced Git revision throughout your application. Enable the `tokio`
feature so iced can execute the command runner's asynchronous tasks.

```toml
[dependencies]
iced = { git = "https://github.com/iced-rs/iced", features = ["tokio"] }
iced_command_runner = { git = "https://github.com/Yiheng-Yu/iced-command-runner", branch = "dev" }

[patch.crates-io]
iced_core = { git = "https://github.com/iced-rs/iced" }
iced_widget = { git = "https://github.com/iced-rs/iced" }
iced_winit = { git = "https://github.com/iced-rs/iced" }
```

## Getting started

Store a `CommandRunner` in your application, wrap its events in your message type,
and return its mapped task from `update`. No subscription is needed.
The current iced view API accepts `impl Widget<Message>`. Use `.boxed()` when you
need to convert a widget to an `iced::Element`.

```rust,no_run
use iced::{Task, Widget, widget::container};
use iced_command_runner::{CommandRunner, Event, create_runner, terminal_container};

#[derive(Debug, Clone)]
enum Message {
    Runner(Event),
}

struct App {
    runner: CommandRunner,
}

impl App {
    fn new() -> Self {
        Self { runner: create_runner("echo", ["Hello from iced!"]) }
    }

    fn view(&self) -> impl Widget<Message> + '_ {
        container(terminal_container(
            &self.runner, Message::Runner, "Run", "Clear history",
        ))
        .padding(10)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Runner(event) => self.runner.update(event).map(Message::Runner),
        }
    }
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view).run()
}
```

`terminal_container` returns an opaque widget. Wrap it in `container` to add
padding, alignment, sizing, or container styles.

## Custom layouts

Combine `run_button`, `clear_buffer_button`, `CommandRunner::crate_view`, and
`status_bar` in your own layout. Both button helpers accept widget content and
return buttons that support iced's button styling API.

```rust
use iced::{Widget, widget::{button, column}};
use iced_command_runner::{CommandRunner, Event, run_button, clear_buffer_button, status_bar};

#[derive(Debug, Clone)]
enum Message {
    Runner(Event),
}

fn view(runner: &CommandRunner) -> impl Widget<Message> + '_ {
    column![
        run_button("Run!", &runner.status, Message::Runner).style(button::primary),
        clear_buffer_button("Wipe!", &runner.status, Message::Runner),
        runner.crate_view(Message::Runner),
        status_bar::<Message>(&runner.status),
    ]
    .spacing(5)
}
```

## Styling

Configure the runner's prompt, text size, background, borders, and font using its
builder methods. Background and border callbacks receive the iced theme.

```rust
use iced::{Background, Color, border::Border, font::Font};
use iced_command_runner::create_runner;

let runner = create_runner("echo", ["hello"])
    .prompt("yourname@localhost:")
    .text_size(13.0)
    .background(|_theme| Background::Color(Color::BLACK))
    .border_idle(|_theme| Border::default())
    .font(Font::MONOSPACE);
```

## Examples

Run the examples from the repository root:

```sh
cargo run --example default_layout
cargo run --example custom_layout
cargo run --example multiple_window
cargo run --example pbar
cargo run --example stream
```

The progress bar and stream demos require Python available as `python`.
The progress bar demo also requires the Python `tqdm` package.
`multiple_window` shows two runners side by side in one application window.

## Custom output rendering

For alternative rendering, read the runner's public `buffer`. Each item is a
`Terminal::StdIn`, `Terminal::StdOut`, `Terminal::StdErr`, or `Terminal::Error`.
`Terminal::as_str()` returns its text. The runner handles carriage returns (`\r`) by replacing the current output
line, including updates split across pipe reads. This supports progress bar
output while preserving completed lines and the command prompt.

You can also use `run_command`, `run_command_stream_by_buffer`, or
`run_command_stream_by_line` as a backend and handle the output events yourself.

## Similar crates

[iced_term](https://github.com/Harzu/iced_term) provides a terminal emulator.
This crate focuses on executing individual commands and streaming their output;
the child process exits when the command finishes.
