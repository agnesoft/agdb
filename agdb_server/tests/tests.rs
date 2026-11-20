mod api;
mod runner;
mod server;
mod test_server;

use std::process::ExitCode;

fn main() -> ExitCode {
    let tests: Vec<&runner::Test> = [api::TESTS, server::TESTS]
        .concat()
        .into_iter()
        .flatten()
        .collect();
    runner::main(&tests, test_server::start)
}
