//! Bounded communication with the compiler plugin explicitly selected by the CLI.
use molecule_codegen::{ast::Ast, ir::Capabilities, IntermediateFormat};
use process_wrap::std::{StdChildWrapper, StdCommandWrap};
use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const MAX_DIAGNOSTIC: u64 = 64 * 1024;
const MAX_GENERATED_CODE: u64 = 64 * 1024 * 1024;

struct Process {
    child: Box<dyn StdChildWrapper>,
    workers: Vec<thread::JoinHandle<()>>,
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
        let _ = self.child.inner_mut().wait();
        let deadline = Instant::now() + Duration::from_millis(250);
        for worker in self.workers.drain(..) {
            while !worker.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(1));
            }
            if worker.is_finished() {
                let _ = worker.join();
            }
        }
    }
}

enum Event {
    Stdout(std::io::Result<Vec<u8>>),
    Stderr(std::io::Result<Vec<u8>>),
    Input(std::io::Result<()>),
}

fn read_limited(reader: impl Read, limit: u64) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("response exceeds {limit} bytes"),
        ));
    }
    Ok(bytes)
}

pub fn negotiate(
    plugin: &Path,
    ast: &Ast,
    timeout: Duration,
) -> Result<IntermediateFormat, String> {
    let mut command = Command::new(plugin);
    command.arg("--capabilities");
    let (bytes, _) = communicate(command, None, timeout, MAX_DIAGNOSTIC)
        .map_err(|error| format!("E_PLUGIN_CAPABILITY: {}: {error}", plugin.display()))?;
    Capabilities::from_json(&bytes)?
        .require(ast)
        .map_err(|error| format!("{}: {error}", plugin.display()))
}

/// Generate into a bounded buffer. The caller publishes it only on success.
/// This executes the same explicitly selected plugin as capability negotiation.
pub fn execute(
    plugin: &Path,
    input: Vec<u8>,
    timeout: Duration,
) -> Result<(Vec<u8>, Vec<u8>), String> {
    communicate(
        Command::new(plugin),
        Some(input),
        timeout,
        MAX_GENERATED_CODE,
    )
    .map_err(|error| format!("E_PLUGIN_GENERATION: {}: {error}", plugin.display()))
}

fn communicate(
    mut command: Command,
    input: Option<Vec<u8>>,
    timeout: Duration,
    output_limit: u64,
) -> Result<(Vec<u8>, Vec<u8>), String> {
    let stdin = if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };
    command
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut command: StdCommandWrap = command.into();
    #[cfg(unix)]
    command.wrap(process_wrap::std::ProcessGroup::leader());
    #[cfg(windows)]
    command.wrap(process_wrap::std::JobObject);
    let mut process = Process {
        child: command
            .spawn()
            .map_err(|error| format!("cannot start process: {error}"))?,
        workers: Vec::new(),
    };
    let stdout = process
        .child
        .inner_mut()
        .stdout
        .take()
        .ok_or("missing stdout pipe")?;
    let stderr = process
        .child
        .inner_mut()
        .stderr
        .take()
        .ok_or("missing stderr pipe")?;
    let (send, receive) = mpsc::channel();
    let output_sender = send.clone();
    process.workers.push(
        thread::Builder::new()
            .spawn(move || {
                let _ = output_sender.send(Event::Stdout(read_limited(stdout, output_limit)));
            })
            .map_err(|error| error.to_string())?,
    );
    let error_sender = send.clone();
    process.workers.push(
        thread::Builder::new()
            .spawn(move || {
                let _ = error_sender.send(Event::Stderr(read_limited(stderr, MAX_DIAGNOSTIC)));
            })
            .map_err(|error| error.to_string())?,
    );
    let mut input_complete = input.is_none();
    if let Some(bytes) = input {
        let mut stdin = process
            .child
            .inner_mut()
            .stdin
            .take()
            .ok_or("missing stdin pipe")?;
        process.workers.push(
            thread::Builder::new()
                .spawn(move || {
                    let result = stdin.write_all(&bytes);
                    drop(stdin);
                    let _ = send.send(Event::Input(result));
                })
                .map_err(|error| error.to_string())?,
        );
    }
    let start = Instant::now();
    let mut stdout = None;
    let mut stderr = None;
    loop {
        let Some(remaining) = timeout.checked_sub(start.elapsed()) else {
            return Err("process timed out".into());
        };
        match receive.recv_timeout(remaining.min(Duration::from_millis(20))) {
            Ok(Event::Stdout(result)) => stdout = Some(result.map_err(|error| error.to_string())?),
            Ok(Event::Stderr(result)) => stderr = Some(result.map_err(|error| error.to_string())?),
            Ok(Event::Input(result)) => {
                result.map_err(|error| format!("cannot send IR: {error}"))?;
                input_complete = true;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                thread::sleep(remaining.min(Duration::from_millis(20)))
            }
        }
        if let Some(status) = process
            .child
            .inner_mut()
            .try_wait()
            .map_err(|error| error.to_string())?
        {
            if let (Some(stdout), Some(stderr)) = (&mut stdout, &mut stderr) {
                if !status.success() {
                    return Err(format!(
                        "process exited {status}: {}",
                        String::from_utf8_lossy(stderr)
                    ));
                }
                if input_complete {
                    return Ok((std::mem::take(stdout), std::mem::take(stderr)));
                }
            }
        }
    }
}
