## Overview

Widget for executing terminal commands and stream outputs

## Getting started

Instance of CommandRunner itself does not work on its own, as you will need to incorporate it into other iced widgets to make it work. The `iced_command_runner` module comes with some complimentary helper functions that provides a good enough (at least for me) out-of-box solution. Feel free referring to the `demo.rs` in the provided `examples` for more details.

Here's a step-by-step guide of doing it:

### **Step 1.** Add `Event` and `CommandRunner` to your application

Add `Event` to your application's `Message`:

```rust
use iced_command_runner::{
    CommandRunner,
    event::Event
};

#[derive(Clone)]
enum Message {
    Runner(Event),  // used for passing command execution events
    ..  // the rest of messages in your module
}
```

Add `CommandRunner`:

```rust
struct App {
    runner: CommandRunner,
    ..  // the rest of the app
}
```

To init a new `CommandRunner` instance, use `CommandRunner::new` or `create_runner`.
Example:

```rust
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

### **Step 2.** Set up`view()`

#### Streaming output

`CommandRunner` impls `crate_view()` method that renders a simulated termianl window that streams termianl output as they got produced:

```rust
pub fn view<'a>(&'a self) -> Element<'a, Message> {
    ...
    let terminal_window = self.runner.crate_view(
        Message::Runner
        );
    ...
}
```

#### Helper functions

##### Triggering command execution

Command execution is triggered by `Event::Execute`. You need to manually implement widgets to trigger this event and pass the event to to the `CommandRunner` instance via the `CommandRunner::update` methdod.

You can also use the (very handy!) function that comes with `iced_command_runner`, `run_button`:

```rust
// in your view() function:
let trigger = run_button(
    widget::text("Run!"),   // or just plain &str
    &self.runner.status,  // current status of the command runner 
    Message::Runner // wraps Event inside your application `Message`
    );
```

###### Executation status

`iced_command_runner` also comes with another (very handy!! and very beautiful!!) widget that displays current execution status:

```rust
let status_bar = status_bar(&self.runner.status);
```

### **Step 3.** Set up `update()`

Pass `event` back to the runner in your `update()`.

```rust
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

Styling configurations is pretty straightforward:

```rust
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

```rust
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
}
```

#### Example use case: stream python's `tqdm` progress bar correctly

[tqdm](https://github.com/tqdm/tqdm) is a python package that prints progress bars to the terminal. Instead of running as a separate proess, progress bar in `tqdm` is printed to `stderr` (not `stdout`!) via carriage return `\r`.

Assume you had setup your application like this:

```rust
use iced_command_runner::{
    CommandRunner,
    event::{Event, Terminal}
};

#[derive(Clone)]
enum Message {
    Runner(Event),  // used for passing command execution events
    ..  // the rest of messages in your module
}

// and your app contains `CommandRunner` field called `runner`
struct App {
    runner: CommandRunner,
    ..  // the rest of the app
}
```

`event::Terminal` provides a helper function `starts_with_carriage_return` that detects if the termianl output starts with `\r` carriage return. You can add an extra if-else check In your application's `update` function to mimick `\r` behaviour by overwriting the last output in your `buffer`:

In your application's `update` function:

```rust
...
match message {
    ...
    Message::Runner(event) => {
        match event {
            Event::Stream(output) => {
                if output.starts_with_carriage_return() {
                    self.buffer.truncate(self.buffer.len()-1); // remove last item
                    self.runner.update(event)
                }
            },
            _ => self.runner.update(event)
        }
    }
    ...
```

### Redirect terminal output elsewhere

You can use `CommandRunner` solely as a backend that simply executes command in a child process, and provides [`Stream`](https://docs.rs/futures-core/0.3.32/futures_core/stream/trait.Stream.html) that pushes data to your application.

The function that responsible for creating `Task::Stream` is via the private function `create_stream`. You can `impl` your own code that looks like:

```rust
use iced_command_runner::CommandRunner;

impl CommandRunner{
    fn my_stream(&self) -> Task<Message> {
        let output_stream = self.create_stream();
        // do whatever you want here..
    }

    ..
    pub update(&self, Message) -> Task<Message> {
        // handle incoming messages as usual
    }
}
```

## Similar crates

The main motivation for writing this is being able to create a GUI app that runs various pre-written CLI scripts in parallel. An alternative choice would be the [iced_term](https://github.com/Harzu/iced_term). It is a terminal emulator widget that more or less does the same job. The main difference between `iced_term` and the current crate is that `iced_command_runner` does not use `subscribe`. It spawns a child process, execute the command and shuts itself down after running it.

The good thing about this apporach is you don't have some command line process that constantly runs in the background. This means it's much easier to spawn lots and lots and lots of child processes that gets executed in parallel. The bad thing about this approach is also, you don't have some command line process that constantly runs in the background, which can be *really* handy some times. For example, implementing `stdin` turned out to be a bit of a headacahe.