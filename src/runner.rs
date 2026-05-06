use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct RunningProcess {
    pub child: Child,
    pub output: Arc<Mutex<String>>,
}

impl RunningProcess {
    pub fn start(command: &str, shell: &str) -> std::io::Result<Self> {
        let mut child = Command::new(shell)
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let output_clone = Arc::clone(&output);

        thread::spawn(move || {
            let mut buf = String::new();
            if let Some(stdout) = stdout {
                let mut reader = BufReader::new(stdout);
                loop {
                    buf.clear();
                    match reader.read_line(&mut buf) {
                        Ok(0) => break,
                        Ok(_) => {
                            if let Ok(mut out) = output_clone.lock() {
                                out.push_str(&buf);
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
        });

        let output_clone = Arc::clone(&output);
        thread::spawn(move || {
            let mut buf = String::new();
            if let Some(stderr) = stderr {
                let mut reader = BufReader::new(stderr);
                loop {
                    buf.clear();
                    match reader.read_line(&mut buf) {
                        Ok(0) => break,
                        Ok(_) => {
                            if let Ok(mut out) = output_clone.lock() {
                                out.push_str(&buf);
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
        });

        Ok(Self { child, output })
    }

    pub fn get_output(&self) -> String {
        self.output.lock().unwrap().clone()
    }

    pub fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }

    pub fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }

    pub fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.child.wait()
    }

    pub fn id(&self) -> u32 {
        self.child.id()
    }
}

pub fn send_signal(pid: u32, sig: nix::sys::signal::Signal) -> Result<(), nix::Error> {
    let pid = nix::unistd::Pid::from_raw(pid as i32);
    nix::sys::signal::kill(pid, sig)
}
