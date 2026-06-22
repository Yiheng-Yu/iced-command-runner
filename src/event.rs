//! messages for streaming terminal command execution
use iced::widget::text_editor::Action as EditorAction;

/// Events
#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    /// Triggers terminal command
    Execute,
    /// flag for letting CommandRunner know it's command spawing was successful
    Spawing,
    /// data produced by the terminal
    Stream(Terminal),
    /// user interactions on the terminal output window, mainly used for making output texts selectable 
    /// (i.e., can select text -> copy -> paste elsewhere)
    EditorAction(EditorAction),
    /// Exit success
    ExitSuccess,
    /// Exit failure with error message
    ExitError(String),
}

/// Outputs from the terminal
#[derive(Clone, PartialEq, Debug)]
pub enum Terminal {
    StdIn(String),  // TODO: add support to pass stdin to an running process?
    StdOut(String),
    StdErr(String),
    Error(String),
}

impl Terminal {
    pub fn as_str(&self) -> &str {
        match self {
            Terminal::StdIn(s) => s,
            Terminal::StdOut(s) => s,
            Terminal::StdErr(s) => s,
            Terminal::Error(s) => s,
        }
    }

    pub fn starts_with_carriage_return(&self) -> bool {
        match self {
            Terminal::StdIn(s) => s.starts_with("\r"),
            Terminal::StdOut(s) => s.starts_with("\r"),
            Terminal::StdErr(s) => s.starts_with("\r"),
            Terminal::Error(s) => s.starts_with("\r"),
        }
    }
}
