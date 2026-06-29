
`iced_command_runner` is an iced widget for used for executing terminal commands and stream outputs. 

Links to documentation:

- [Crates.io](https://crates.io/crates/iced_command_runner)

- [Doc for the latest release](https://docs.rs/iced_command_runner/latest/iced_command_runner/)


Note that this is still a bit of work-in-progress. Current API is done and won't change (at least too much) in the forseeable future. There might be some small bugs here and there but the crate itself should be functional for majorities of the use case. They will get fixed one by one in my free time.

If you found any issues, please do feel free to [submit them](https://github.com/Yiheng-Yu/iced-command-runner/issues) or do some [pull requests](https://github.com/Yiheng-Yu/iced-command-runner/pulls) on your fixes/ improvements, thanks in advance.

`iced` it self is also in active development. I will try my best to udpate this crate trakcing the most recent stable release of `iced` as soon as I could.

Cheers.

## Overview

The main stuff of the `iced_command_runner` crate is `CommandRunner`, it is a struct that holds data for `program` and `args` to execute, as well as `buffer` that stores data streamed from previous and currently running processes. `CommandRunner` provides a default `create_view()` function that renders terminal output in ways similar to a shell.

Command execution is trigged by `Event::Execute` enum.

Instance of CommandRunner itself does not work on its own, as you will need to incorporate it into other iced widgets to make it work.

Feel free referring to the `demo.rs` in the provided `examples` for a working example.

## Getting started

Here is the easiest way of doing it:

### **Step 1.** Add `Event` and `CommandRunner` to your application

Add `Event` to your application's `Message`:

```rust{ignore}
use iced_command_runner::{
    CommandRunner,
    event::Event
};

#[derive(Clone)]
enum Message {
    Runner(Event),  // used for passing command execution events
    ...  // the rest of messages in your module
}
```

Add `CommandRunner`:

```rust{ignore}
struct App {
    runner: CommandRunner,
    ..  // the rest of the app
}
```

To init a new `CommandRunner` instance, use `CommandRunner::new` or `create_runner`.
Example:

```rust{ignore}
use iced_command_runner::create_runner;

impl App {
    pub fn new(
        ..  // your codes
        command: impl Into<String>, 
        args: impl IntoIterator<Item = impl Into<String>>
    ) -> Self {
        let runner = create_runner(command, args);  // or CommandRunner::new(command, args)

        Self {
            ...  // your codes
            runner: runner
        }
    }
}
```

### **Step 2.** Set up your `view()` function

The `iced_command_runner` module comes with a complimentary helper function `terminal_container` that provides a good enough (at least for me) out-of-box solution that renders a default layout. A demo of the default layout can be found in `examples/default_layout` (`cargo run --example default_layout`).

To implement, simply put this in your `App`'s `view`:

```rust{ignore}
...
let terminal_window = terminal_container(&runner, Message::Runner, "Run", "Clear history")
.spacing(..)
.align_x(..)
.align_y(..)
.width(..)
...
```

Alternatively, you can set up your own layout as follow:

#### Setting up your own layout

The `iced_command_runner` comes with several widgets that meant to work together, and you can arrange/ use them however you like. A demo of an example custom layout can be found in `examples/custom_layout` (`cargo run --example custom_layout`).

Here's a walkthrough of the widgets:

##### Live stream terminal output

`CommandRunner` impls `crate_view()` method that renders a simulated termianl window that streams termianl output as they got produced:

```rust{ignore}
pub fn view<'a>(&'a self) -> Element<'a, Message> {
    ...
    let terminal_window = self.runner.crate_view(
        Message::Runner
        );
    ...
}
```

##### Triggering command execution

Command execution is triggered by `Event::Execute`. You need to manually implement widgets to trigger this event and pass the event to to the `CommandRunner` instance via the `CommandRunner::update` methdod.

You can also use the (very handy!) function that comes with `iced_command_runner`, `run_button`:

```rust{ignore}
// in your view() function:
let trigger = run_button(
    widget::text("Run!"),   // or just plain &str
    &self.runner.status,  // current status of the command runner 
    Message::Runner // wraps Event inside your application `Message`
    )
    .style(..);  // function that returns button::Style
```

###### Clear buffer history

`clear_buffer_button()` returns a button that clears history stored in `CommandRunner`:

```rust{ignore}
use iced_command_runner::clear_buffer_button;
...
    // in your view() function:
    let clear_history = clear_buffer_button(
        widget::text("Wipe!"),   // or just plain &str
        &self.runner.status,  // current status of the command runner 
        Message::Runner // wraps Event inside your application `Message`
        )
        .style(..);  // function that returns button::Style
```

###### Executation status

`iced_command_runner` also comes with another (very handy!! and very beautiful!!) widget that displays current execution status:

```rust{ignore}
let status_bar = status_bar::<Message>(&self.runner.status);
```

### **Step 3.** Set up `update()`

Simply pass `event` back to the runner in your `update()` function, note that:

1) the `update()` function NEEDS TO return `iced::Task` in order for this module to work

2) you need to map the output of `runner.update()` back into your `Message`

Example:

```rust{ignore}
pub fn update(
    &mut self, 
    message: Message
) -> iced::Task<Message> {  // note that the `update()` function NEEDS TO return `iced::Task` in order for this module to work.
    match message {
        Message::Runner(event) => 
            self.runner.update(event)  // pass event to your runner
            .map(Message::Runner),  //! don't forget to map the output of `runner.update()` back into your Message::Runner
        // the rest of the update()
        ... 
    }
}
```

That's it, there's nothing else you need to do, you don't need to set up `subscribe` etc. Just `run()` your application and see the result.

## Styling

Two button functions (`clear_buffer_button` and `run_button`) return an `iced::widget::button::Button` instance that allows you to style them however you like.

The `CommandRunner` styling is pretty similar too:

```rust{ignore}
let runner = create_runner(command, args)
    .prompt("yourname@localhost:")
    .text_size(8.0)
    .background(..)
    .border_idle(..)
    .font(..);
```

## Dev notes

### Alternative ways of rendering terminal output

If you don't like currently rendering setups and wishing to render things differently, you can implement your own `view()` functions that renders terminal outputs stored in the `buffer` field:

```rust{ignore}
let content: Vec<Element<'_, Message>> = runner.buffer.iter().map(
    |output| {
        match output {
            // implement your own styling functions here
            Terminal::StdIn(text) => ...,
            Terminal::StdOut(text) => ...,
            Terminal::StdErr(text) => ...,
            Terminal::Error(text) => ...,
        }
    }
    ).collect();
```

#### Carriage return

By default, `CommandRunner`'s `update()` function would automatically remove previous message if the new message starts with carriage return (`\r`). This is particularly useful to correctly printing progress bars (like the one in python's `tqdm`). The current solution *works* but not as ideal. Proper handling this is planned for future releases.

### Redirect terminal output elsewhere

You can use `CommandRunner` solely as a backend that simply executes command in a child process, and provides [`Stream`](https://docs.rs/futures-core/0.3.32/futures_core/stream/trait.Stream.html) that pushes data to your application. You can trigger the command execution & data streaming in your `update()` method and handling rendeirng etc. as you wish.

## Similar crates

The main motivation for writing this is being able to create a GUI app that runs various pre-written CLI scripts in parallel. An alternative choice would be the [iced_term](https://github.com/Harzu/iced_term). It is a terminal emulator widget that more or less does the same job. The main difference between `iced_term` and the current crate is that `iced_command_runner` does not use `subscribe`. It spawns a child process, execute the command and shuts itself down after running it.

The good thing about this apporach is you don't have some command line process that constantly runs in the background. This means it's much easier to spawn lots and lots and lots of child processes that gets executed in parallel. The bad thing about this approach is also, you don't have some command line process that constantly runs in the background, which can be *really* handy some times. For example, implementing `stdin` turned out to be a bit of a headacahe.

## Future (to do list)

- Set up alternative displaying options (i.e., use text/ rich_text instead of iced::text_editor)
- Set up 'terminate' button that terminates/ kills currently running process.
- Set up support for `stdin`