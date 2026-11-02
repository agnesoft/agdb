use agbench::Bench;
use agplatform::CommandExt;
use agplatform::Env;
use agplatform::Platform;
use agplatform::TestDir;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

const DATA_LEN: usize = 1024 * 1024;

fn main() -> ExitCode {
    agbench::main(&[
        run,
        run_timeout,
        run_path,
        output,
        input,
        run_async,
        output_async,
        input_async,
    ])
}

/// A temporary directory with a [`DATA_LEN`] file set as the current
/// directory with the platform created in it, so that `std` and
/// `agplatform` run commands in the same place. Restores the current
/// directory when dropped.
struct Fixture<P> {
    platform: P,
    current_dir: PathBuf,
    _dir: TestDir,
}

fn fixture() -> Fixture<impl Platform> {
    let dir = TestDir::temp();
    std::fs::write(dir.path().join("data"), data()).expect("data should be writable");
    let current_dir = std::env::current_dir().expect("current dir should be accessible");
    std::env::set_current_dir(dir.path()).expect("test dir should be enterable");

    Fixture {
        platform: agplatform::platform(),
        current_dir,
        _dir: dir,
    }
}

impl<P> Drop for Fixture<P> {
    fn drop(&mut self) {
        // WHY: Windows cannot remove the current directory.
        let _ = std::env::set_current_dir(&self.current_dir);
    }
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime should be buildable")
}

fn data() -> Vec<u8> {
    b"a\n".repeat(DATA_LEN / 2)
}

#[cfg(unix)]
fn noop() -> Command {
    Command::new("true")
}

#[cfg(windows)]
fn noop() -> Command {
    let mut command = Command::new("cmd");
    command.args(["/C", "exit 0"]);
    command
}

#[cfg(unix)]
fn print_data() -> Command {
    let mut command = Command::new("cat");
    command.arg("data");
    command
}

#[cfg(windows)]
fn print_data() -> Command {
    let mut command = Command::new("cmd");
    command.args(["/C", "type data"]);
    command
}

#[cfg(unix)]
fn echo_stdin() -> Command {
    Command::new("cat")
}

#[cfg(windows)]
fn echo_stdin() -> Command {
    let mut command = Command::new("findstr");
    command.arg("^");
    command
}

fn run(bench: &mut Bench) {
    bench
        .benchmark("process/run", fixture)
        .fail_factor(1.5)
        .target("std", |_| noop().output().unwrap())
        .target("agplatform", |fx| {
            noop().run_blocking(fx.platform.process()).unwrap()
        })
        .run();
}

fn run_timeout(bench: &mut Bench) {
    bench
        .benchmark("process/run/timeout", fixture)
        .fail_factor(1.5)
        .target("std", |_| noop().output().unwrap())
        .target("agplatform", |fx| {
            noop()
                .timeout(Duration::from_secs(60))
                .run_blocking(fx.platform.process())
                .unwrap()
        })
        .run();
}

/// The `PATH` of the current process with the fixture directory appended,
/// which still finds the programs but differs from the inherited one.
fn changed_path() -> String {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let dirs = std::env::split_paths(&path).chain([std::env::temp_dir()]);
    std::env::join_paths(dirs)
        .expect("paths should be joinable")
        .into_string()
        .expect("PATH should be valid UTF-8")
}

fn run_path(bench: &mut Bench) {
    bench
        .benchmark("process/run/path", || {
            let fx = fixture();
            fx.platform.env().set_var("PATH", changed_path()).unwrap();
            (fx, changed_path())
        })
        .fail_factor(1.5)
        .target("std", |(_, path)| {
            noop().env("PATH", path).output().unwrap()
        })
        .target("agplatform", |(fx, _)| {
            noop().run_blocking(fx.platform.process()).unwrap()
        })
        .run();
}

fn output(bench: &mut Bench) {
    bench
        .benchmark("process/output", fixture)
        .fail_factor(1.5)
        .target("std", |_| print_data().output().unwrap().stdout.len())
        .target("agplatform", |fx| {
            print_data()
                .run_blocking(fx.platform.process())
                .unwrap()
                .stdout
                .len()
        })
        .run();
}

fn input(bench: &mut Bench) {
    bench
        .benchmark("process/input", || (fixture(), data()))
        .fail_factor(1.5)
        .target("std", |(_, data)| {
            let mut child = echo_stdin()
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let mut stdin = child.stdin.take().unwrap();
            let data = data.clone();
            let writer = std::thread::spawn(move || stdin.write_all(&data));
            let output = child.wait_with_output().unwrap();
            writer.join().unwrap().unwrap();
            output.stdout.len()
        })
        .target("agplatform", |(fx, data)| {
            echo_stdin()
                .input(data.as_slice())
                .run_blocking(fx.platform.process())
                .unwrap()
                .stdout
                .len()
        })
        .run();
}

fn run_async(bench: &mut Bench) {
    bench
        .benchmark("process/run_async", || (fixture(), runtime()))
        .fail_factor(1.5)
        .target("tokio", |(_, runtime)| {
            runtime
                .block_on(async { tokio::process::Command::from(noop()).output().await })
                .unwrap()
        })
        .target("agplatform", |(fx, runtime)| {
            runtime
                .block_on(noop().run_async(fx.platform.process()))
                .unwrap()
        })
        .run();
}

fn output_async(bench: &mut Bench) {
    bench
        .benchmark("process/output_async", || (fixture(), runtime()))
        .fail_factor(1.5)
        .target("tokio", |(_, runtime)| {
            runtime
                .block_on(async { tokio::process::Command::from(print_data()).output().await })
                .unwrap()
                .stdout
                .len()
        })
        .target("agplatform", |(fx, runtime)| {
            runtime
                .block_on(print_data().run_async(fx.platform.process()))
                .unwrap()
                .stdout
                .len()
        })
        .run();
}

fn input_async(bench: &mut Bench) {
    bench
        .benchmark("process/input_async", || (fixture(), runtime(), data()))
        .fail_factor(1.5)
        .target("tokio", |(_, runtime, data)| {
            runtime.block_on(async {
                let mut child = tokio::process::Command::from(echo_stdin())
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap();
                let mut stdin = child.stdin.take().unwrap();
                let data = data.clone();
                let writer = tokio::spawn(async move { stdin.write_all(&data).await });
                let output = child.wait_with_output().await.unwrap();
                writer.await.unwrap().unwrap();
                output.stdout.len()
            })
        })
        .target("agplatform", |(fx, runtime, data)| {
            runtime
                .block_on(
                    echo_stdin()
                        .input(data.as_slice())
                        .run_async(fx.platform.process()),
                )
                .unwrap()
                .stdout
                .len()
        })
        .run();
}
