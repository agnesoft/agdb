use agplatform::BlockingRequestBuilderExt;
use agplatform::CommandExt;
use agplatform::Env;
use agplatform::Fs;
use agplatform::Platform;
use agplatform::Process;
use agplatform::Request;

#[cfg(feature = "testing")]
mod test_platform_test;

#[cfg(unix)]
fn print_var(name: &str) -> std::process::Command {
    let mut command = std::process::Command::new("sh");
    command.args(["-c", &format!("echo \"${{{name}-unset}}\"")]);
    command
}

#[cfg(windows)]
fn print_var(name: &str) -> std::process::Command {
    let mut command = std::process::Command::new("cmd");
    command.args([
        "/C",
        &format!("if defined {name} (echo %{name}%) else (echo unset)"),
    ]);
    command
}

#[cfg(unix)]
fn exit_3() -> std::process::Command {
    let mut command = std::process::Command::new("sh");
    command.args(["-c", "exit 3"]);
    command
}

#[cfg(windows)]
fn exit_3() -> std::process::Command {
    let mut command = std::process::Command::new("cmd");
    command.args(["/C", "exit 3"]);
    command
}

mod platform {
    use super::*;

    #[test]
    fn platform_should_expose_public_api() {
        agplatform::Error::fs("fs error");
        agplatform::Error::env("env error");
        agplatform::Error::process("process error");
        agplatform::Error::request("request error");

        let platform = agplatform::platform();
        platform.env();
        platform.env();
        platform.fs();
        platform.fs();
        platform.process();
        platform.process();
        platform.request();
        platform.request();
    }

    #[test]
    fn platform_should_be_shared_with_threads_and_tasks() {
        fn assert_send<F: std::future::Future + Send>(future: F) -> F {
            future
        }

        fn share<P: Platform + 'static>(platform: P) {
            let platform = std::sync::Arc::new(platform);
            let shared = platform.clone();

            std::thread::spawn(move || shared.fs().exists("Cargo.toml").unwrap())
                .join()
                .unwrap();

            // WHY: holding the interfaces across an await in a generic
            // function requires them to be Send + Sync, as `tokio::spawn`
            // does.
            let task = std::pin::pin!(assert_send(async move {
                let (env, fs, process, request) = (
                    platform.env(),
                    platform.fs(),
                    platform.process(),
                    platform.request(),
                );
                std::future::ready(()).await;
                assert!(env.current_dir().join("Cargo.toml").exists());
                assert!(fs.exists("Cargo.toml").unwrap());
                std::process::Command::new(env!("CARGO"))
                    .arg("--version")
                    .run_blocking(process)
                    .unwrap();
                request.blocking_client().unwrap();
            }));

            assert!(
                task.poll(&mut std::task::Context::from_waker(std::task::Waker::noop()))
                    .is_ready()
            );
        }

        share(agplatform::platform());
    }

    #[test]
    fn platform_should_run_commands() {
        let platform = agplatform::platform();

        let output = std::process::Command::new(env!("CARGO"))
            .arg("--version")
            .timeout(std::time::Duration::from_secs(60))
            .run_blocking(platform.process())
            .unwrap();

        assert!(output.stdout_str().unwrap().starts_with("cargo "));
    }

    #[test]
    fn platform_should_pass_env_vars_to_processes() {
        // GIVEN a variable set and another one removed from the environment
        let platform = agplatform::platform();
        platform.env().set_var("AGPLATFORM_SET", "value").unwrap();
        platform
            .env()
            .set_var("AGPLATFORM_REMOVED", "value")
            .unwrap();
        platform.env().remove_var("AGPLATFORM_REMOVED");

        // WHEN printing them from child processes
        let set = print_var("AGPLATFORM_SET")
            .run_blocking(platform.process())
            .unwrap();
        let removed = print_var("AGPLATFORM_REMOVED")
            .run_blocking(platform.process())
            .unwrap();

        // THEN the child processes see the environment
        assert_eq!(set.stdout_str().unwrap().trim(), "value");
        assert_eq!(removed.stdout_str().unwrap().trim(), "unset");
    }

    #[test]
    fn platform_should_pass_runs_to_process_interceptors() {
        // GIVEN an interceptor collecting failures on the platform
        use agplatform::ProcessCommand;
        use agplatform::interceptor::{BoxFuture, Interceptor, Next};

        #[derive(Clone, Default)]
        struct Failures(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

        impl Interceptor<ProcessCommand> for Failures {
            fn handle<'a>(
                &'a self,
                mut next: Next<'a, ProcessCommand>,
            ) -> BoxFuture<'a, agplatform::Result<agplatform::CommandOutput>> {
                Box::pin(async move {
                    let result = next.attempt().await;

                    match &result {
                        Ok(output) if !output.success => self
                            .0
                            .lock()
                            .unwrap()
                            .push(format!("failed with {:?}", output.status)),
                        Err(error) => self.0.lock().unwrap().push(error.description().to_string()),
                        Ok(_) => {}
                    }

                    result
                })
            }
        }

        let failures = Failures::default();
        let platform = agplatform::platform();
        platform.process().add_interceptor(failures.clone());

        // WHEN a command fails to start and another one exits with 3
        let _ = std::process::Command::new("agplatform_missing_program")
            .run_blocking(platform.process());
        let _ = exit_3().run_blocking(platform.process());

        // THEN the interceptor sees both failures
        let failures = failures.0.lock().unwrap().clone();
        assert_eq!(failures.len(), 2, "{failures:?}");
        assert!(failures[0].starts_with("'agplatform_missing_program': "));
        assert_eq!(failures[1], "failed with Some(3)");
    }

    #[test]
    fn platform_should_send_requests() {
        // GIVEN a server answering one request
        use std::io::{BufRead, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/ping", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = std::io::BufReader::new(&stream);
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap() > 2 {
                line.clear();
            }
            (&stream)
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 4\r\nconnection: close\r\n\r\npong")
                .unwrap();
        });
        let platform = agplatform::platform();

        // WHEN sending a request to it
        let output = platform
            .request()
            .blocking_client()
            .unwrap()
            .get(url)
            .send_blocking(platform.request())
            .unwrap();

        // THEN the response is returned
        assert_eq!(output.text().unwrap(), "pong");
        server.join().unwrap();
    }

    #[test]
    fn platform_should_send_requests_through_proxy_of_env() {
        // GIVEN a proxy answering one request set in the platform environment
        use std::io::{BufRead, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let proxy = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = std::io::BufReader::new(&stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap() > 2 {
                line.clear();
            }
            (&stream)
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\nconnection: close\r\n\r\n")
                .unwrap();
            request_line
        });
        let platform = agplatform::platform();
        for key in ["NO_PROXY", "no_proxy", "REQUEST_METHOD"] {
            platform.env().remove_var(key);
        }
        platform.env().set_var("HTTP_PROXY", proxy).unwrap();

        // WHEN sending a request to another host
        platform
            .request()
            .blocking_client()
            .unwrap()
            .get("http://example.invalid/path")
            .send_blocking(platform.request())
            .unwrap();

        // THEN it goes through the proxy
        assert_eq!(
            server.join().unwrap(),
            "GET http://example.invalid/path HTTP/1.1\r\n"
        );
    }
}
