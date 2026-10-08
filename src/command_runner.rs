use crate::{ Argument, event::{ Event, Terminal }, run_command, StreamMode };

// rustfmt::skip
use iced::widget::text_editor; /* rustfmt::skip  <- formatter would auto iced::widget::text_editor{self,}, which imported text_editor as a file instead of a function */
use iced::{
    Background,
    Color,
    Element,
    Length,
    Pixels,
    Task,
    Widget,
    border::Border,
    font::{ Family, Font, Stretch, Style as FontStyle, Weight },
    stream::channel,
    theme::Theme,
    widget::{
        container,
        scrollable,
        text,
        text::LineHeight,
        text_editor::Content,
        void,
    },
};
use log::warn;
use std::cmp::{ max, min };


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

impl Status {
    /// list of status that should be considered 'running'
    pub fn running(&self) -> [Status; 2] {
        [Status::Initialize, Status::Running]
    }

    /// check if some members is 'running' -> this includes `Running` and `Initialize`
    pub fn is_running(status: &Status) -> bool {
        match status {
            Status::Failed(_) => false,
            Status::Idle => false,
            Status::Running => true,
            Status::Initialize => true,
        }
    }
}

/// CommandRunner that executes terminal command, and stream the output to its buffer.
/// Instance of CommandRunner itself does not work on its own, as you will need to incorporate it with other iced widgets to make it work.
///
/// Example usage:
/// ```rust
/// use iced_command_runner::create_runner;
///
/// let runner = create_runner("echo", ["hiii"])
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

    mode: StreamMode,
    /// style configurations. Check iced_command_runner::Style for more details
    style: Style,
    /// used for iced text_editor widget, for displaying seleectable texts
    content: Content,
    // The current output line is independent of the user's editor cursor.
    output_line: Option<usize>,
    overwrite_line: bool,
}

impl CommandRunner {
    /// create new CommandRunner instance, example:
    /// ```rust
    /// use iced_command_runner::CommandRunner;
    /// let runner = CommandRunner::new("echo", ["hiii"])
    /// .text_size(13.0);
    /// ```
    pub fn new(
        command: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>
    ) -> Self {
        Self {
            command: Argument::new(command, args),
            buffer: Vec::new(),
            status: Status::Idle,
            mode: StreamMode::Buffer(256),
            style: Style::default(),
            content: Content::new(),
            output_line: None,
            overwrite_line: false,
        }
    }

    /// create new instace with no empty args
    pub fn new_no_args(command: impl Into<String>) -> Self {
        Self {
            command: Argument {
                program: command.into(),
                args: Vec::new(),
            },
            mode: StreamMode::Buffer(256),
            buffer: Vec::new(),
            status: Status::Idle,
            style: Style::default(),
            content: Content::new(),
            output_line: None,
            overwrite_line: false,
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
    /// Crates a mocked terminal window to display message received in `self.buffer`
    pub fn crate_view<'a, Message>(
        &'a self,
        on_update: impl (Fn(Event) -> Message) + 'a
    ) -> Element<'a, Message>
        where Message: Clone + 'a
    {
        if self.style.selectable_text {
            selectable_terminal_window(self, on_update)
        } else {
            plain_terminal_window(self)
        }
    }

    // ------------------------------------------------------------------
    // Updating internal states
    // ------------------------------------------------------------------
    fn format_command(&self) -> String {
        format!("{} {} {}\n", self.style.prompt, self.command.program, self.command.args.join(" "))
    }

    fn update_editor_content(&mut self) {
        let mut text = String::new();
        for msg in &self.buffer {
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(msg.as_str());
        }
        self.content = Content::with_text(&text);
        self.content.perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
    }

    fn push_to_buffer(&mut self, to_push: Terminal) {
        if matches!(to_push, Terminal::StdIn(_)) {
            self.output_line = None;
            self.overwrite_line = false;
            self.buffer.push(to_push);
        } else {
            let mut empty_line = to_push.clone();
            empty_line.text_mut().clear();
            for ch in to_push.as_str().chars() {
                if ch == '\r' {
                    self.overwrite_line = true;
                    continue;
                }
                let index = *self.output_line.get_or_insert_with(|| {
                    self.buffer.push(empty_line.clone());
                    self.buffer.len() - 1
                });
                let line = self.buffer[index].text_mut();
                // Wait for text after CR: CRLF must preserve the finished line.
                if self.overwrite_line && ch != '\n' {
                    line.clear();
                }
                self.overwrite_line = false;
                line.push(ch);
                if ch == '\n' {
                    self.output_line = None;
                }
            }
        }
        self.update_editor_content();
    }

    fn create_stream(&mut self) -> Task<Event> {
        let runner = self.command.clone();
        let mode = self.mode.clone();
        let streamer = channel(1024, |messenger| run_command(runner, messenger, mode));
        Task::stream(streamer)
    }
    pub fn is_running(&self) -> bool {
        Status::is_running(&self.status)
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
                    }
                }
            }

            Event::ClearBuffer => {
                self.buffer = Vec::new();
                self.output_line = None;
                self.overwrite_line = false;
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
                // self.push_to_buffer(Terminal::Error(err_message));
                self.status = Status::Failed(err_message);
                Task::none()
            }
        }
    }

    // ------------------------------------------------------------------
    // command running
    // ------------------------------------------------------------------
    /// Update argument for the command to run. i.e:
    /// ```rust
    /// use iced_command_runner::{CommandRunner, Event};
    /// let mut runner = CommandRunner::new("echo", ["hello!"]);
    /// runner.set_args(["hi"]);
    /// // Return this task from your application update, mapping it to your message.
    /// let task = runner.update(Event::Execute);
    /// ```
    pub fn set_args(&mut self, args: impl IntoIterator<Item = impl Into<String>>) {
        let new_argument = args
            .into_iter()
            .map(|s| s.into())
            .collect::<Vec<String>>();
        self.command.args = new_argument;
    }

    // ------------------------------------------------------------------
    // styling & Configurations
    // ------------------------------------------------------------------
    /// stream data line by line
    /// calling tokio BufReader::read_line at the backend
    pub fn stream_mode_line(mut self) -> Self {
        self.mode = StreamMode::Line;
        self
    }
    
    /// stream data by filling the buffer
    /// /// calling tokio BufReader::read at the backend
    pub fn stream_mode_buffer(mut self, size: usize) -> Self {
        self.mode = StreamMode::Buffer(size);
        self
    }

    /// when self.mode is StreamMode::Line -> does nothing
    pub fn channel_buffer_size(mut self, size: usize) -> Self {
        match &self.mode {
            StreamMode::Line => self,
            StreamMode::Buffer(_) => {
                self.mode = StreamMode::Buffer(size);
                self
            }
        }
    }

    /// creates different borders based on current state of the runner
    pub fn dynamic_border(&self, theme: &Theme) -> Border {
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

    pub fn selectable_text(mut self, is_selectable: bool) -> Self {
        self.style.selectable_text = is_selectable;
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

    /// overwrites `self.style.min_lines` if it's larger than current `num_lines`
    pub fn max_lines(mut self, num_lines: usize) -> Self {
        if num_lines < self.style.min_lines {
            self.style.min_lines = num_lines;
        }

        self.style.max_lines = num_lines;
        self
    }

    /// overwrites `self.style.max_lines` if it's smaller than current `num_lines`
    pub fn min_lines(mut self, num_lines: usize) -> Self {
        if num_lines > self.style.max_lines {
            self.style.max_lines = num_lines;
        }

        self.style.min_lines = num_lines;
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
    /// Max number of lines to display in the screen before starting to scroll,
    /// this also sets max_height of the terminal window.
    pub max_lines: usize,

    /// Minimum number of lines to display in the screen
    /// This also sets the minimum height of the terminal window.
    /// Needs to be > 0
    pub min_lines: usize,

    /// width of the terminal window
    pub width: Length,
    /// background colour for the terminal window
    pub background: fn(&Theme) -> Background,

    /// toggle on/ off for rendering selectable/ non-selectable terminal window
    /// because currently rendered iced::widget::text is not selectable,
    /// uses text_editor widget instead when selectable_text as 'true'
    pub selectable_text: bool,

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
        Self {
            prompt: " >".to_string(),
            width: Length::Fill,
            max_lines: 12,
            min_lines: 3,
            background: |theme| {
                let palette = theme.palette();
                Background::Color(palette.background.weakest.color)
            },
            selectable_text: true,
            border_idle: |theme| {
                let palette = theme.palette();
                Border {
                    color: palette.primary.base.color,
                    width: 1.0,
                    ..Default::default()
                }
            },
            border_running: |theme| {
                let palette = theme.palette();
                Border {
                    color: palette.primary.base.color,
                    width: 1.5,
                    ..Default::default()
                }
            },
            border_error: |theme| {
                let palette = theme.palette();
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
    pub fn calc_height(&self, n_lines: usize) -> Length {
        let text_size: Pixels = self.text_size.into();
        let n_lines: Pixels = (n_lines as f32).into();
        let height_pixels: Pixels = text_size * n_lines;
        Length::from(height_pixels)
    }
}

pub fn selectable_terminal_window<'a, Message>(
    runner: &'a CommandRunner,
    on_update: impl (Fn(Event) -> Message) + 'a
) -> Element<'a, Message>
    where Message: Clone + 'a
{
    if runner.buffer.is_empty() && runner.style.min_lines == 0 {
        return void().boxed();
    }

    // a string styling thingy:
    // when there was just 1 line to print
    // horizontal scrollbar WILL COVER that one single line, if it gets shown
    // to disable this we'd also need to make this a special case
    let n_lines = runner.buffer.len();
    let n_lines = if n_lines == 1 { 2 } else { n_lines };
    let n_lines = max(runner.style.min_lines, n_lines);
    let n_lines = min(n_lines, runner.style.max_lines);
    let editor_height = runner.style.calc_height(n_lines);

    let editor = text_editor(&runner.content)
        .wrapping(iced_core::text::Wrapping::None)
        .placeholder("")
        .font(runner.style.font)
        .line_height(runner.style.line_height)
        .on_action(move |action| on_update(Event::EditorAction(action)))
        .style(|theme: &Theme, _status: iced::widget::text_editor::Status| {
            let palette = theme.palette();
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
        .height(editor_height)
        .size(runner.style.text_size);

    let content = scrollable(editor).auto_scroll(true).anchor_bottom();

    container(content)
        .height(Length::Shrink)
        .width(runner.style.width)
        .style(|theme| container::Style {
            background: Some((runner.style.background)(theme)),
            border: runner.dynamic_border(theme),
            ..Default::default()
        })
        .boxed()
}

pub fn plain_terminal_window<'a, Message>(runner: &'a CommandRunner) -> Element<'a, Message>
    where Message: Clone + 'a
{
    if runner.buffer.len() == 0 && runner.style.min_lines == 0 {
        return iced::widget::space().height(0.0).boxed();
    }

    let font = runner.style.font;
    let text_size = runner.style.text_size;
    let line_height = runner.style.line_height;

    let content = text(runner.content.text())
        .font(font)
        .size(text_size)
        .line_height(line_height)
        .wrapping(iced_core::text::Wrapping::Word);

    // Horizontal scrollbars in iced::scrollable blocks last line of text
    // if there's only 1 line to display & no vertical scrollbar
    // so the actual min_lines for scrollable to work would be 3
    // if in future this gets fixed then uhh yeah would save a lot of effort
    // (p.s: setting spacing() for horizontal bars also DOES NOT WORK)
    let current_buffer_size = runner.buffer.len();

    let bottom_padding = if current_buffer_size <= 3 {
        runner.style.text_size * (1.0 + 0.25 * (current_buffer_size as f32))
    } else {
        runner.style.text_size * 0.75
    };

    let content = container(content)
        .height(Length::Shrink)
        .width(Length::Fill)
        .padding(iced::Padding {
            top: 3.0,
            right: 1.0,
            bottom: bottom_padding, // bottom_padding, // so horizontal won't overlay text
            left: 1.0,
        });

    let content = if current_buffer_size > runner.style.max_lines {
        let n_lines = max(runner.style.min_lines, current_buffer_size);
        let n_lines = min(n_lines, runner.style.max_lines);
        let max_height = runner.style.calc_height(n_lines);
        // TODO: ADD SCROLLABLE STYLING OPTIONS TO THE STYLE STRUCT
        scrollable::Scrollable
            ::with_direction(content, scrollable::Direction::Both {
                vertical: scrollable::Scrollbar::default().margin(0.0),
                horizontal: scrollable::Scrollbar::default().margin(0.0),
            })
            .height(max_height)
            .width(Length::Fill)
            .auto_scroll(true)
            .anchor_bottom()
    } else {
        let max_height = runner.style.calc_height(current_buffer_size);
        scrollable::Scrollable
            ::with_direction(
                content,
                scrollable::Direction::Horizontal(scrollable::Scrollbar::default().margin(0.0))
            )
            .height(max_height)
            .width(Length::Fill)
            .auto_scroll(false)
    };

    container(content)
        .height(Length::Shrink)
        .width(runner.style.width)
        .padding(2.0)
        .style(|theme| container::Style {
            background: Some((runner.style.background)(theme)),
            border: runner.dynamic_border(theme),
            ..Default::default()
        })
        .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_command() {
        let runner = CommandRunner::new("echo", ["hello!"]);
        assert_eq!(runner.program(), "echo");
        assert_eq!(runner.args(), ["hello!"]);
        assert_eq!(runner.status, Status::Idle);
    }

    #[test]
    fn test_terminal_views() {
        let runner = CommandRunner::new("echo", ["hello!"]);
        let _: Element<'_, Event> = runner.crate_view(|event| event);
        let _: Element<'_, Event> = crate::terminal_container(
            &runner, |event| event, "Run", "Clear",
        ).boxed();
        let runner = runner.selectable_text(false);
        let _: Element<'_, Event> = runner.crate_view(|event| event);
    }

    #[test]
    fn progress_updates_replace_one_output_line() {
        let mut runner = CommandRunner::new("python", ["demo.py"]);
        runner.push_to_buffer(Terminal::StdIn("$ python demo.py\n".into()));
        runner.push_to_buffer(Terminal::StdErr("\rProgress: 0%".into()));
        // A user moving the editor cursor must not affect output replacement.
        runner.content.perform(text_editor::Action::SelectAll);
        runner.push_to_buffer(Terminal::StdErr("\rProgress: 50%\rProgress: 100%\n".into()));
        assert_eq!(runner.buffer.len(), 2);
        assert_eq!(runner.buffer[0], Terminal::StdIn("$ python demo.py\n".into()));
        assert_eq!(runner.buffer[1], Terminal::StdErr("Progress: 100%\n".into()));
        assert_eq!(runner.content.text(), "$ python demo.py\nProgress: 100%\n");
    }

    #[test]
    fn progress_updates_handle_split_chunks_and_crlf() {
        let mut runner = CommandRunner::new_no_args("echo");
        for chunk in ["\rProgress: ", "0%", "\r", "Progress: 100%", "\r", "\nNext", " line\n"] {
            runner.push_to_buffer(Terminal::StdOut(chunk.into()));
        }
        assert_eq!(runner.buffer, vec![
            Terminal::StdOut("Progress: 100%\n".into()),
            Terminal::StdOut("Next line\n".into()),
        ]);
        let _ = runner.update(Event::ClearBuffer);
        runner.push_to_buffer(Terminal::StdOut("fresh".into()));
        assert_eq!(runner.buffer, vec![Terminal::StdOut("fresh".into())]);
    }

    #[test]
    fn test_runner_status() {
        let mut runner = CommandRunner::new("echo", ["hello!"]);

        // don't update with Event::update and it should be fine
        let _ = runner.update(Event::Spawing);
        assert_eq!(runner.is_running(), true);
        let check = &runner.status != &Status::Idle;
        assert_eq!(check, true);

        let tmp_msg = "".to_string();
        let _ = runner.update(Event::Stream(Terminal::Error(tmp_msg.clone())));
        assert_eq!(runner.is_running(), true);
        let check = &runner.status != &Status::Idle;
        assert_eq!(check, true);

        let _ = runner.update(Event::Stream(Terminal::StdIn(tmp_msg.clone())));
        assert_eq!(runner.is_running(), true);
        let check = &runner.status != &Status::Idle;
        assert_eq!(check, true);

        let _ = runner.update(Event::Stream(Terminal::StdOut(tmp_msg.clone())));
        assert_eq!(runner.is_running(), true);
        let check = &runner.status != &Status::Idle;
        assert_eq!(check, true);

        let _ = runner.update(Event::Stream(Terminal::StdErr(tmp_msg.clone())));
        assert_eq!(runner.is_running(), true);
        let check = &runner.status != &Status::Idle;
        assert_eq!(check, true);

        let _ = runner.update(Event::ClearBuffer);
        assert_eq!(runner.is_running(), true);
        let check = &runner.status != &Status::Idle;
        assert_eq!(check, true);

        let _ = runner.update(Event::ExitSuccess);
        assert_eq!(runner.is_running(), false);
        let check = &runner.status == &Status::Idle;
        assert_eq!(check, true);

        let _ = runner.update(Event::ExitError(tmp_msg.clone()));
        assert_eq!(runner.is_running(), false);
        let check = &runner.status != &Status::Idle;
        assert_eq!(check, true);
    }
}
