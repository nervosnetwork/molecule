//! Bounded communication with the compiler plugin explicitly selected by the CLI.
use molecule_codegen::{ast::Ast, ir::Capabilities, IntermediateFormat};
use process_wrap::std::{StdChildWrapper, StdCommandWrap};
use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Output, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const MAX_DIAGNOSTIC: u64 = 64 * 1024;
const MAX_GENERATED_CODE: u64 = 64 * 1024 * 1024;

struct Process {
    child: Box<dyn StdChildWrapper>,
    workers: Vec<thread::JoinHandle<()>>,
    kill_on_drop: bool,
}
impl Drop for Process {
    fn drop(&mut self) {
        if self.kill_on_drop {
            let _ = self.child.start_kill();
            let _ = self.child.wait();
        }
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

pub enum PluginFormat {
    Wire(IntermediateFormat),
    WithSources(IntermediateFormat),
}

fn capability_format(plugin: &Path, bytes: &[u8], ast: &Ast) -> Result<IntermediateFormat, String> {
    Capabilities::from_json(bytes)
        .and_then(|caps| caps.require(ast))
        .map_err(|err| format!("{}: {err}", plugin.display()))
}

/// The v1 format probe also lets a new plugin request owners for a plain import
/// graph. A v1 format name, or a plugin lacking the probe, retains the wire ABI.
pub fn plain_format(plugin: &Path, ast: &Ast, timeout: Duration) -> Result<PluginFormat, String> {
    let mut command = Command::new(plugin);
    command.arg("--format");
    if ast.imports().is_empty() {
        command.env_remove(crate::SOURCE_IR_OFFER);
    } else {
        command.env(crate::SOURCE_IR_OFFER, "2");
    }
    let reply = communicate(command, None, timeout, MAX_DIAGNOSTIC)
        .map_err(|err| format!("E_PLUGIN_CAPABILITY: {}: {err}", plugin.display()))?;
    if reply.status.success() {
        let text = std::str::from_utf8(&reply.stdout).unwrap_or("").trim();
        if text.starts_with('{') {
            return capability_format(plugin, &reply.stdout, ast).map(PluginFormat::WithSources);
        }
        if let Ok(format) = IntermediateFormat::try_from(text) {
            return Ok(PluginFormat::Wire(format));
        }
    }
    Ok(PluginFormat::Wire(IntermediateFormat::JSON))
}

pub fn negotiate(
    plugin: &Path,
    ast: &Ast,
    timeout: Duration,
) -> Result<IntermediateFormat, String> {
    let mut command = Command::new(plugin);
    command.arg("--capabilities");
    let (bytes, _) = communicate(command, None, timeout, MAX_DIAGNOSTIC)
        .and_then(successful)
        .map_err(|error| format!("E_PLUGIN_CAPABILITY: {}: {error}", plugin.display()))?;
    capability_format(plugin, &bytes, ast)
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
    .and_then(successful)
    .map_err(|error| format!("E_PLUGIN_GENERATION: {}: {error}", plugin.display()))
}

fn successful(reply: Output) -> Result<(Vec<u8>, Vec<u8>), String> {
    if reply.status.success() {
        Ok((reply.stdout, reply.stderr))
    } else {
        Err(format!(
            "process exited {}: {}",
            reply.status,
            String::from_utf8_lossy(&reply.stderr)
        ))
    }
}

fn communicate(
    mut command: Command,
    input: Option<Vec<u8>>,
    timeout: Duration,
    output_limit: u64,
) -> Result<Output, String> {
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
        kill_on_drop: true,
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
    let mut input_result = input.is_none().then_some(Ok(()));
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
    } else {
        drop(send);
    }
    let start = Instant::now();
    let mut stdout = None;
    let mut stderr = None;
    let mut exit_status = None;
    loop {
        let Some(remaining) = timeout.checked_sub(start.elapsed()) else {
            return Err("process timed out".into());
        };
        let disconnected = match receive.recv_timeout(remaining.min(Duration::from_millis(20))) {
            Ok(Event::Stdout(result)) => {
                stdout = Some(result.map_err(|error| error.to_string())?);
                false
            }
            Ok(Event::Stderr(result)) => {
                stderr = Some(result.map_err(|error| error.to_string())?);
                false
            }
            // Defer reporting stdin write failures (e.g. BrokenPipe) until the child
            // exits and stderr is collected, so an early plugin diagnostic is preserved.
            Ok(Event::Input(result)) => {
                input_result = Some(result);
                false
            }
            Err(mpsc::RecvTimeoutError::Timeout) => false,
            Err(mpsc::RecvTimeoutError::Disconnected) => true,
        };
        // Wait until all pipes close before reaping so an exited leader pins its
        // Unix PGID until `start_kill()` runs if descendants still hold pipes open.
        let io_done =
            (stdout.is_some() && stderr.is_some() && input_result.is_some()) || disconnected;
        if io_done && exit_status.is_none() {
            exit_status = process
                .child
                .try_wait()
                .map_err(|error| error.to_string())?;
            if exit_status.is_some() {
                process.kill_on_drop = false;
            }
        }
        if let Some(status) = exit_status {
            if let (Some(stdout), Some(stderr)) = (&mut stdout, &mut stderr) {
                if let Some(result) = input_result.take() {
                    if status.success() {
                        result.map_err(|error| format!("cannot send IR: {error}"))?;
                    }
                    return Ok(Output {
                        status,
                        stdout: std::mem::take(stdout),
                        stderr: std::mem::take(stderr),
                    });
                }
            }
            if disconnected {
                return Err("plugin I/O worker exited unexpectedly".into());
            }
        } else if disconnected {
            thread::sleep(remaining.min(Duration::from_millis(5)));
        }
    }
}
