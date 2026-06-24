use crate::{
    Argument,
    event::{Event, Terminal},
    run_command,
};

// rustfmt::skip
use iced::widget::text_editor; // text_editor::{self, Content} would import text_editor the module instead of text_editor the funciton
use iced::{
    Background, Color, Element, Length, Pixels, Task,
    border::Border,
    font::{Family, Font, Stretch, Style as FontStyle, Weight},
    stream::channel,
    theme::Theme,
    widget::{
        container, 
        scrollable, 
        text::LineHeight, 
        text_editor::{Content, Action, Edit}
    },
};
use log::warn;

/// Command execution status
#[derive(Clone, PartialEq, Debug)]
pub enum Status {
    /// ready to run
    Idle,
    /// spawing
    Initialize,
    /// currently running command
    Running,
    /// ExitError with error message
    Failed(String),
}

/// CommandRunner that executes terminal command, and stream the output to its buffer.
/// Instance of CommandRunner itself does not work on its own, as you will need to incorporate it with other iced widgets to make it work. 
/// 
/// Example usage:
/// ```rust
/// use iced_cmd_runner::create_runner;
/// 
/// let runner = create_runner::new("echo", ["hiii"])
/// .text_size(13.0);
/// ```
#[derive(Clone, Debug)]
pub struct CommandRunner {
    /// command to run
    pub command: Argument,
    /// message streamed from the spawned process
    pub buffer: Vec<Terminal>,
    /// current status i.e., running, idle etc.,
    pub status: Status,
    /// style configurations. Check iced_command_runner::Style for more details
    style: Style,
    /// used for iced text_editor widget, for displaying seleectable texts
    content: Content,
}

impl CommandRunner {
    /// create new CommandRunner instance, example:
    /// ```rust
    /// use iced_cmd_runner::CommandRunner;
    /// let runner = CommandRunner::new("echo", ["hiii"])
    /// .text_size(13.0);
    /// ```
    pub fn new(command: impl Into<String>, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            command: Argument::new(command, args),
            buffer: Vec::new(),
            status: Status::Idle,
            style: Style::default(),
            content: Content::new(),
        }
    }

    /// create new instace with no empty args
    pub fn new_no_args(command: impl Into<String>) -> Self {
        Self {
            command: Argument {program: command.into(), args: Vec::new()},
            buffer: Vec::new(),
            status: Status::Idle,
            style: Style::default(),
            content: Content::new(),
        }
    }

    /// Arguments to run terminal commands with
    pub fn args(&self) -> Vec<String> {
        self.command.args.clone()
    }

    /// program to run
    pub fn program(&self) -> &str {
        &self.command.program
    }

    // ------------------------------------------------------------------
    // Drawing
    // ------------------------------------------------------------------
    fn wrap_inside_scrollable<'a, Message>(&'a self, content: impl Into<Element<'a, Message>>) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        let content: Element<'a, Message> = scrollable(content.into())
            .auto_scroll(true)
            .anchor_bottom()
            .into();

        self.wrap_inside_container(content)
    }

    fn wrap_inside_container<'a, Message>(&'a self, content: impl Into<Element<'a, Message>>) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        container(content)
            .width(self.style.width)
            .height(self.style.calc_height())
            .style(|theme| container::Style {
                background: Some((self.style.background)(theme)),
                border: self.border_style(theme),
                ..Default::default()
            })
            .into()
    }

    /// Crates a mocked terminal window to display message received in `self.buffer`
    pub fn crate_view<'a, Message>(&'a self, on_update: impl Fn(Event) -> Message + 'a) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {   
        let editor = text_editor(&self.content)
            .placeholder("")
            .font(self.style.font)
            .line_height(self.style.line_height)
            .on_action(move |action| on_update(Event::EditorAction(action)))
            .style(|theme: &Theme, _status: iced::widget::text_editor::Status| {
                let palette = theme.extended_palette();
                text_editor::Style {
                    background: Background::Color(Color::TRANSPARENT),
                    border: Border {
                        width: 0.0,
                        ..Default::default()
                    },
                    placeholder: palette.secondary.base.color,
                    value: palette.background.base.text,
                    selection: palette.primary.weak.color,
                }
            })
            .size(self.style.text_size);

        // optional render as scrollable
        match self.style.max_lines {
            Some(_) => self.wrap_inside_scrollable(editor),
            None => self.wrap_inside_container(editor),
        }
    }

    // ------------------------------------------------------------------
    // Updating internal states
    // ------------------------------------------------------------------
    pub fn is_running(&self) -> bool {
        self.status != Status::Idle
    }

    fn format_command(&self) -> String {
        format!("{} {} {}\n", &self.style.prompt, &self.command.program, &self.command.args.join(" "))
    }

    fn update_editor_content(&mut self) {
        let mut content: Vec<String> = Vec::new();
        for msg in self.buffer.iter() {
            match msg {
                Terminal::StdIn(msg) => content.push(format!("{} {}", &self.style.prompt, msg)),
                _ => content.push(msg.as_str().to_string()),
            }
        }
        self.content = Content::with_text(content.join("").trim());
    }

    fn push_to_buffer(&mut self, to_push: Terminal) {
        self.buffer.push(to_push.clone());
        self.content.perform(
            Action::Edit(Edit::Paste(to_push.as_str().to_string().into())
        ));
    }

    fn create_stream(&mut self) -> Task<Event> {
        let runner = self.command.clone();
        let streamer = channel(1024, |messenger| run_command(runner, messenger));
        Task::stream(streamer)
    }

    pub fn update(&mut self, event: Event) -> Task<Event> {
        match event {
            // Start streaming data
            Event::Execute => {
                if !self.is_running() {
                    self.status = Status::Initialize;
                    self.push_to_buffer(Terminal::StdIn(self.format_command()));
                    self.create_stream()
                } else {
                    // Techanically for most cases it's okay, it's just that for now
                    // CommandRunner.buffer is just one single Vec<> therefor cannot distinguish outputs from different spawned child processes.
                    // This can be easily worked around by using some sort of Hashmap instead (i.e, Hashmap<int, Vec<Terminal>>)
                    // (if you really want to)
                    warn!("Cannot invoke a new instance whilst current one is still running!");
                    Task::none()
                }
            }

            // Allow users select & copy terminal outputs, disabling everythingh else
            Event::EditorAction(action) => {
                // Only block actual editing actions
                match action {
                    // Block editing actions (these contain the actual editing operations)
                    text_editor::Action::Edit(_) => Task::none(),
                    // Allow all other actions (movement, selection, clicking, scrolling, etc.)
                    _ => {
                        self.content.perform(action);
                        Task::none()
                    },
                }
            }

            Event::ClearBuffer => {
                self.buffer = Vec::new();
                self.update_editor_content();
                Task::none()
            }

            Event::Spawing => {
                self.status = Status::Running;
                Task::none()
            }

            // storing data from the stream
            Event::Stream(msg) => {
                self.push_to_buffer(msg);
                Task::none()
            }

            Event::ExitSuccess => {
                self.status = Status::Idle;
                Task::none()
            }

            Event::ExitError(err_message) => {
                self.status = Status::Failed(err_message);
                Task::none()
            }
        }
    }

    // ------------------------------------------------------------------
    // command running
    // ------------------------------------------------------------------
    /// Update argument for the command to run. i.e:
    /// ```
    /// use iced_cmd_runner::CommandRunner;
    /// let runner = CommandRunner::new("echo", ["hello!"]);  // receives Terminal::StdOut("hello!\n")
    /// runner.set_args(["hi"]);
    /// runner.update(Event::Execute).await; // receives Terminal::StdOut("hi\n")
    /// ```
    pub fn set_args(&mut self, args: impl IntoIterator<Item = impl Into<String>>) {
        let new_argument = args.into_iter().map(|s| s.into()).collect::<Vec<String>>();
        self.command.args = new_argument;
    }

    // ------------------------------------------------------------------
    // styling
    // ------------------------------------------------------------------
    fn border_style(&self, theme: &Theme) -> Border {
        match self.status {
            Status::Idle => (self.style.border_idle)(theme),
            Status::Initialize => (self.style.border_running)(theme),
            Status::Running => (self.style.border_running)(theme),
            Status::Failed(_) => (self.style.border_error)(theme),
        }
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.style.prompt = prompt.into();
        self
    }

    pub fn background(mut self, background_fn: fn(&Theme) -> Background) -> Self {
        self.style.background = background_fn.into();
        self
    }

    pub fn border_idle(mut self, border_fn: fn(&Theme) -> Border) -> Self {
        self.style.border_idle = border_fn.into();
        self
    }

    pub fn border_running(mut self, border_fn: fn(&Theme) -> Border) -> Self {
        self.style.border_running = border_fn.into();
        self
    }

    pub fn border_error(mut self, border_fn: fn(&Theme) -> Border) -> Self {
        self.style.border_error = border_fn.into();
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.style.font = font;
        self
    }

    pub fn text_size(mut self, text_size: f32) -> Self {
        self.style.text_size = text_size;
        self
    }

    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.style.line_height = line_height.into();
        self
    }

    pub fn num_lines(mut self, num_lines: u32) -> Self {
        self.style.max_lines = Some(num_lines);
        self
    }

    pub fn height(mut self, length: impl Into<Length>) -> Self {
        self.style.height = length.into();
        self.style.max_lines = None;
        self
    }

    pub fn width(mut self, length: impl Into<Length>) -> Self {
        self.style.width = length.into();
        self
    }
}

/// Styling options for CommandRunner
#[derive(Clone, Debug)]
pub struct Style {
    /// shell prompt, i.e., the `>>>` thingy in python, the `username@location:` thingy in bash
    pub prompt: String,
    /// terminal text size, default: 12.0
    pub text_size: f32,
    /// Line height, same as iced::LineHeight::default()
    pub line_height: LineHeight,
    /// Max number of lines to display in the screen.
    /// When set, overrides the 'height' attribute and uses line_height x max_lines as the height of the terminal window.
    pub max_lines: Option<u32>,
    /// width of the terminal window
    pub width: Length,
    /// Height of the terminal window. Overridden by max_lines if it was not None
    pub height: Length,
    /// background colour for the terminal window
    pub background: fn(&Theme) -> Background,

    /// widget border when CommandRunner.status is Status::Idle (ready to run)
    pub border_idle: fn(&Theme) -> Border,
    /// widget border when CommandRunner is running commands
    pub border_running: fn(&Theme) -> Border,
    /// widget border when command exited with error
    pub border_error: fn(&Theme) -> Border,
    /// default iced::font::Family::Monospace
    pub font: Font,
}


impl Default for Style {
    fn default() -> Self {
        // zch-like shell prompt
        let cwd = std::env::current_dir().unwrap_or_default();
        let cwd = if cwd.components().count() > 3 {
            let full_path = std::env::current_dir()
                .unwrap_or_default()
                .iter()
                .rev()
                .take(3)
                .map(|s| s.to_str().unwrap_or_default())
                .collect::<Vec<&str>>()
                .join("/");

            format!("..{}", full_path)
        } else {
            cwd
            .to_str()
            .unwrap_or_default()
            .to_string()
        };
        
        Self {
            prompt: format!("{} >", cwd),
            width: Length::Fill,
            max_lines: Some(12),
            height: (300.0).into(),
            background: |theme| {
                let palette = theme.extended_palette();
                Background::Color(palette.background.weakest.color)
            },

            border_idle: |theme| {
                let palette = theme.extended_palette();
                Border {
                    color: palette.primary.base.color,
                    width: 1.0,
                    ..Default::default()
                }
            },
            border_running: |theme| {
                let palette = theme.extended_palette();
                Border {
                    color: palette.primary.base.color,
                    width: 1.5,
                    ..Default::default()
                }
            },
            border_error: |theme| {
                let palette = theme.extended_palette();
                Border {
                    color: palette.danger.base.color,
                    width: 1.5,
                    ..Default::default()
                }
            },

            font: Font {
                family: Family::Monospace,
                weight: Weight::Normal,
                style: FontStyle::Normal,
                stretch: Stretch::Normal,
            },
            text_size: 12.0,
            line_height: LineHeight::Relative(1.3), // iced default
        }
    }
}

impl Style {
    pub fn calc_height(&self) -> Length {
        match self.max_lines {
            Some(n_lines) => {
                let text_size: Pixels = self.text_size.into();
                let n_lines: Pixels = n_lines.into();
                let height_pixels: Pixels = text_size * n_lines;
                Length::from(height_pixels)
            }
            None => self.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_command() {
        let runner = CommandRunner::new("echo", ["hello!"]);
    }
}
