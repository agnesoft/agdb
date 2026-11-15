use agbench::Bench;
use agplatform::BlockingRequestBuilderExt;
use agplatform::Platform;
use agplatform::Request;
use agplatform::RequestBuilderExt;
use bytes::Bytes;
use std::collections::VecDeque;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::ExitCode;

const DATA_LEN: usize = 1024 * 1024;

fn main() -> ExitCode {
    agbench::main(&[
        get,
        get_data,
        get_data_sink,
        post_data,
        get_async,
        get_data_async,
        get_data_sink_async,
        post_data_async,
    ])
}

/// A local HTTP/1.1 server with keep-alive and the platform whose default
/// clients both targets use, so that they share the connection pool.
struct Fixture<P> {
    platform: P,
    url: String,
}

fn fixture() -> Fixture<impl Platform> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let url = format!("http://{}", listener.local_addr().expect("address"));
    // WHY: glibc maps the bodies anew, faulting in each page, until it
    // frees a larger mapping, so the targets would differ by their order.
    drop(std::hint::black_box(vec![0_u8; 4 * DATA_LEN]));

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            std::thread::spawn(move || serve(stream));
        }
    });

    Fixture {
        platform: agplatform::platform(),
        url,
    }
}

/// Answers `/data` with [`DATA_LEN`] bytes and anything else with an empty
/// body until the client closes the connection.
fn serve(stream: TcpStream) {
    // WHY: otherwise Nagle's algorithm with delayed ACKs adds ~40 ms per response.
    let _ = stream.set_nodelay(true);
    let mut reader = BufReader::new(&stream);
    let mut writer = &stream;
    let data = data();

    loop {
        let mut head = String::new();

        while reader.read_line(&mut head).unwrap_or(0) > 2 {}

        if head.is_empty() {
            return;
        }

        let length = head
            .to_lowercase()
            .lines()
            .find_map(|line| line.strip_prefix("content-length: ")?.parse().ok())
            .unwrap_or(0);
        let mut body = vec![0; length];
        let _ = reader.read_exact(&mut body);
        let body: &[u8] = if head.starts_with("GET /data ") {
            &data
        } else {
            &[]
        };
        let mut response =
            format!("HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n", body.len()).into_bytes();
        response.extend_from_slice(body);
        let _ = writer.write_all(&response);
    }
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime should be buildable")
}

fn data() -> Vec<u8> {
    vec![b'a'; DATA_LEN]
}

/// The request bodies of the last iterations, each freed by a later setup
/// rather than by a target.
///
/// WHY: reqwest frees a body it was given on its own thread, while
/// agplatform keeps it on the caller's thread until the request ends, to
/// retry it, so only agplatform would be measured freeing it.
#[derive(Default)]
struct Held(VecDeque<Bytes>);

impl Held {
    /// The inputs of `iteration_setup` prepared at once at most by agbench,
    /// so that a body freed is of an iteration already measured.
    const BODIES: usize = 64;

    fn body(&mut self) -> Bytes {
        if self.0.len() == Self::BODIES {
            self.0.pop_front();
        }

        let body = Bytes::from(data());
        self.0.push_back(body.clone());
        body
    }
}

fn get(bench: &mut Bench) {
    bench
        .benchmark("request/get", || {
            let fx = fixture();
            let client = fx.platform.request().blocking_client().unwrap();
            (fx, client)
        })
        .fail_factor(1.5)
        .target("reqwest", |(fx, client)| {
            client.get(&fx.url).send().unwrap().bytes().unwrap().len()
        })
        .target("agplatform", |(fx, client)| {
            client
                .get(&fx.url)
                .send_blocking(fx.platform.request())
                .unwrap()
                .body
                .len()
        })
        .run();
}

fn get_data(bench: &mut Bench) {
    bench
        .benchmark("request/get/1MiB", || {
            let fx = fixture();
            let client = fx.platform.request().blocking_client().unwrap();
            let url = format!("{}/data", fx.url);
            (fx, client, url)
        })
        .fail_factor(1.5)
        .target("reqwest", |(_, client, url)| {
            client
                .get(url.as_str())
                .send()
                .unwrap()
                .bytes()
                .unwrap()
                .len()
        })
        .target("agplatform", |(fx, client, url)| {
            client
                .get(url.as_str())
                .send_blocking(fx.platform.request())
                .unwrap()
                .body
                .len()
        })
        .run();
}

fn get_data_sink(bench: &mut Bench) {
    bench
        .benchmark("request/get/1MiB/sink", || {
            let fx = fixture();
            let client = fx.platform.request().blocking_client().unwrap();
            let url = format!("{}/data", fx.url);
            (fx, client, url)
        })
        .fail_factor(1.5)
        .target("reqwest", |(_, client, url)| {
            client
                .get(url.as_str())
                .send()
                .unwrap()
                .copy_to(&mut std::io::sink())
                .unwrap()
        })
        .target("agplatform", |(fx, client, url)| {
            let mut received = 0;
            client
                .get(url.as_str())
                .send_blocking_with_sink(fx.platform.request(), &mut |(), chunk: &[u8]| {
                    received += chunk.len();
                    Ok(())
                })
                .unwrap();
            received
        })
        .run();
}

fn post_data(bench: &mut Bench) {
    bench
        .benchmark("request/post/1MiB", || {
            let fx = fixture();
            let client = fx.platform.request().blocking_client().unwrap();
            (fx, client, Held::default())
        })
        .fail_factor(1.5)
        .iteration_setup(|(_, _, held)| held.body())
        .target("reqwest", |(fx, client, _), data| {
            client
                .post(&fx.url)
                .body(data)
                .send()
                .unwrap()
                .bytes()
                .unwrap()
                .len()
        })
        .target("agplatform", |(fx, client, _), data| {
            client
                .post(&fx.url)
                .body(data)
                .send_blocking(fx.platform.request())
                .unwrap()
                .body
                .len()
        })
        .run();
}

fn get_async(bench: &mut Bench) {
    bench
        .benchmark("request/get_async", || {
            let fx = fixture();
            let client = fx.platform.request().client().unwrap();
            (fx, client, runtime())
        })
        .fail_factor(1.5)
        .target("reqwest", |(fx, client, runtime)| {
            runtime.block_on(async {
                let response = client.get(&fx.url).send().await.unwrap();
                response.bytes().await.unwrap().len()
            })
        })
        .target("agplatform", |(fx, client, runtime)| {
            runtime
                .block_on(client.get(&fx.url).send_async(fx.platform.request()))
                .unwrap()
                .body
                .len()
        })
        .run();
}

fn get_data_async(bench: &mut Bench) {
    bench
        .benchmark("request/get_async/1MiB", || {
            let fx = fixture();
            let client = fx.platform.request().client().unwrap();
            let url = format!("{}/data", fx.url);
            (fx, client, url, runtime())
        })
        .fail_factor(1.5)
        .target("reqwest", |(_, client, url, runtime)| {
            runtime.block_on(async {
                let response = client.get(url.as_str()).send().await.unwrap();
                response.bytes().await.unwrap().len()
            })
        })
        .target("agplatform", |(fx, client, url, runtime)| {
            runtime
                .block_on(client.get(url.as_str()).send_async(fx.platform.request()))
                .unwrap()
                .body
                .len()
        })
        .run();
}

fn get_data_sink_async(bench: &mut Bench) {
    bench
        .benchmark("request/get_async/1MiB/sink", || {
            let fx = fixture();
            let client = fx.platform.request().client().unwrap();
            let url = format!("{}/data", fx.url);
            (fx, client, url, runtime())
        })
        .fail_factor(1.5)
        .target("reqwest", |(_, client, url, runtime)| {
            runtime.block_on(async {
                let mut response = client.get(url.as_str()).send().await.unwrap();
                let mut received = 0;

                while let Some(chunk) = response.chunk().await.unwrap() {
                    received += chunk.len();
                }

                received
            })
        })
        .target("agplatform", |(fx, client, url, runtime)| {
            let mut received = 0;
            runtime
                .block_on(client.get(url.as_str()).send_async_with_sink(
                    fx.platform.request(),
                    &mut |(), chunk: &[u8]| {
                        received += chunk.len();
                        Ok(())
                    },
                ))
                .unwrap();
            received
        })
        .run();
}

fn post_data_async(bench: &mut Bench) {
    bench
        .benchmark("request/post_async/1MiB", || {
            let fx = fixture();
            let client = fx.platform.request().client().unwrap();
            (fx, client, runtime(), Held::default())
        })
        .fail_factor(1.5)
        .iteration_setup(|(_, _, _, held)| held.body())
        .target("reqwest", |(fx, client, runtime, _), data| {
            runtime.block_on(async {
                let response = client.post(&fx.url).body(data).send().await.unwrap();
                response.bytes().await.unwrap().len()
            })
        })
        .target("agplatform", |(fx, client, runtime, _), data| {
            runtime
                .block_on(
                    client
                        .post(&fx.url)
                        .body(data)
                        .send_async(fx.platform.request()),
                )
                .unwrap()
                .body
                .len()
        })
        .run();
}
