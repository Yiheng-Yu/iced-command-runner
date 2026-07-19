use iced::futures::{channel::mpsc, sink::SinkExt};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::event::{Event, Terminal};


/// controls how data gets streamed
/// StreamMode::Line: Stream output line by line
/// StreamMode::Buffer(usize): stream data by size of the buffer
#[derive(Clone, Debug)]
pub enum StreamMode {
    Line,
    /// size of the buffer for the data stream, this controls the length of &[u8] stored in the backend process
    /// before sending to the CommandRunner instance.
    /// Only used when self.mode is StreamMode::Line
    /// Default: 256
    Buffer(usize)
}

impl Default for StreamMode {
    fn default() -> Self {
        Self::Buffer(256)
    }
}


// ==============================================================================
// Command execution
// ==============================================================================
/// Command line arguments to run
#[derive(Clone, Debug)]
pub struct Argument {
    pub program: String,
    pub args: Vec<String>,
}

impl Argument {
    pub fn new(program: impl Into<String>, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(|n| n.into()).collect(),
        }
    }
}

pub async fn run_command(to_run: Argument, messenger: mpsc::Sender<Event>, mode: StreamMode) {
    match mode {
        StreamMode::Line => {
            run_command_stream_by_line(to_run, messenger).await
        }
        StreamMode::Buffer(buffer_size) => {
            let buffer_size = buffer_size.clone();
            run_command_stream_by_buffer(to_run, buffer_size, messenger).await
        }
    }
}

/// running terminal command and stream the output via `messenger`
pub async fn run_command_stream_by_buffer(to_run: Argument, channel_buffer_size: usize, messenger: mpsc::Sender<Event>) {
    let mut messenger = messenger.clone();
    let child = Command::new(&to_run.program)
        .args(&to_run.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(child) => child,
        Err(e) => {
            let message = Event::Stream(Terminal::Error(e.to_string()));
            let _ = messenger.send(message).await;
            return;
        }
    };

    let _ = messenger.send(Event::Spawing).await;

    let stdout = match child.stdout.take() {
        Some(data) => data,
        None => {
            let message = Event::Stream(Terminal::Error("Unable to spawing stdout".to_string()));
            let _ = messenger.send(message).await;
            return;
        }
    };

    let stderr = match child.stderr.take() {
        Some(data) => data,
        None => {
            let message = Event::Stream(Terminal::Error("Unable to spawing stderr".to_string()));
            let _ = messenger.send(message).await;
            return;
        }
    };

    // // todo: stdin
    // let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(100);
    // let mut sender = stdin_tx.clone();
    // let _ = sender.send(Event::BufferWrite(sender)).await;  // send StdIn back to the runner and do some stuff,,
    let mut stdout_reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);

    let mut stdout_buf = vec![0u8; channel_buffer_size];
    let mut stderr_buf = vec![0u8; channel_buffer_size];

    // unsure if there were any other better ways of doing this, but,,this took me so bloddy long time,,,
    let mut stdout_done = false;
    let mut stderr_done = false;
    loop {
        tokio::select! {
            res = stdout_reader.read(&mut stdout_buf), if !stdout_done => {
                match res {
                    Ok(0) => stdout_done = true,
                    Ok(len) => {
                        let chunk = String::from_utf8_lossy( &stdout_buf[..len] ).to_string();
                        let _ = messenger.send( Event::Stream(Terminal::StdOut(chunk)) ).await;
                    }
                    Err(e) => {
                        let _ = messenger.send( Event::Stream(Terminal::Error(e.to_string())) ).await;
                        break;
                    }
                }
            }

            res = stderr_reader.read(&mut stderr_buf), if !stderr_done => {
                match res {
                    Ok(0) => stderr_done = true,
                    Ok(len) => {
                        let chunk = String::from_utf8_lossy(&stderr_buf[..len]).to_string();
                        let _ = messenger.send( Event::Stream(Terminal::StdErr(chunk)) ).await;
                    }
                    Err(e) => {
                        let _ = messenger.send( Event::Stream(Terminal::Error(e.to_string())) ).await;
                        break;
                    }
                }
            }
        }
        if stdout_done & stderr_done {
            break;
        }
    }

    let status = match child.wait().await {
        Ok(status) => status,
        Err(e) => {
            let _ = messenger.send(Event::Stream(Terminal::Error(e.to_string()))).await;
            return;
        }
    };
    let _ = child.kill().await;

    if status.success() {
        let _ = messenger.send(Event::ExitSuccess).await;
    } else {
        let _ = messenger.send(Event::ExitError(status.to_string())).await;
    }
}

pub async fn run_command_stream_by_line(to_run: Argument, messenger: mpsc::Sender<Event>) {
    let mut messenger = messenger.clone();
    let child = Command::new(&to_run.program)
        .args(&to_run.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(child) => child,
        Err(e) => {
            let message = Event::Stream(Terminal::Error(e.to_string()));
            let _ = messenger.send(message).await;
            return;
        }
    };

    let _ = messenger.send(Event::Spawing).await;

    let stdout = match child.stdout.take() {
        Some(data) => data,
        None => {
            let message = Event::Stream(Terminal::Error("Unable to spawing stdout".to_string()));
            let _ = messenger.send(message).await;
            return;
        }
    };

    let stderr = match child.stderr.take() {
        Some(data) => data,
        None => {
            let message = Event::Stream(Terminal::Error("Unable to spawing stderr".to_string()));
            let _ = messenger.send(message).await;
            return;
        }
    };

    // // todo: stdin
    // let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(100);
    // let mut sender = stdin_tx.clone();
    // let _ = sender.send(Event::BufferWrite(sender)).await;  // send StdIn back to the runner and do some stuff,,
    let mut stdout_reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);

    let mut stdout_buf = String::new();
    let mut stderr_buf = String::new();


    // unsure if there were any other better ways of doing this, but,,this took me so bloddy long time,,,
    let mut stdout_done = false;
    let mut stderr_done = false;
    loop {
        tokio::select! {
            res = stdout_reader.read_line(&mut stdout_buf), if !stdout_done => {
                match res {
                    Ok(0) => stdout_done = true,
                    Ok(_) => {
                        let chunk = stdout_buf.clone();
                        let _ = messenger.send( Event::Stream(Terminal::StdOut(chunk)) ).await;
                        stdout_buf.clear();
                    }
                    Err(e) => {
                        let _ = messenger.send( Event::Stream(Terminal::Error(e.to_string())) ).await;
                        break;
                    }
                }
            }

            res = stderr_reader.read_line(&mut stderr_buf), if !stderr_done => {
                match res {
                    Ok(0) => stderr_done = true,
                    Ok(_) => {
                        let chunk = stderr_buf.clone();
                        let _ = messenger.send( Event::Stream(Terminal::StdErr(chunk)) ).await;
                        stderr_buf.clear()
                    }
                    Err(e) => {
                        let _ = messenger.send( Event::Stream(Terminal::Error(e.to_string())) ).await;
                        break;
                    }
                }
            }
        }
        if stdout_done & stderr_done {
            break;
        }
    }

    let status = match child.wait().await {
        Ok(status) => status,
        Err(e) => {
            let _ = messenger.send(Event::Stream(Terminal::Error(e.to_string()))).await;
            return;
        }
    };
    let _ = child.kill().await;

    if status.success() {
        let _ = messenger.send(Event::ExitSuccess).await;
    } else {
        let _ = messenger.send(Event::ExitError(status.to_string())).await;
    }
}

// ==============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_command_execution() {
        let args = Argument::new("python", ["-u", "-c", "print('hi how are you')"]);

        let (messenger, mut receiver) = mpsc::channel::<Event>(512);
        run_command_stream_by_buffer(args, 256, messenger).await;
        let mut output = String::new();

        loop {
            let data = receiver.recv().await.unwrap();
            match data {
                Event::Stream(res) => match res {
                    Terminal::StdOut(d) => {
                        let d = d.trim();
                        output.push_str(d);
                    }
                    _ => {}
                },
                Event::ExitSuccess => break,
                Event::ExitError(_) => break,
                _ => {}
            }
        }

        assert_eq!(output.trim(), "hi how are you")
    }
}
