use agplatform::BlockingRequestBuilderExt;
use agplatform::CommandExt;
use agplatform::CommandOutput;
use agplatform::Platform;
use agplatform::Request;
use agplatform::RequestOutput;
use agplatform::TransportError;
use agplatform::TransportErrorKind;
use agplatform::reqwest::header::HeaderMap;
use agplatform::test_platform::test_platform;

mod test_platform {
    use super::*;

    #[test]
    fn test_platform_should_expose_public_api() {
        agplatform::TestFs::new();
        agplatform::TestEnv::new();
        agplatform::TestProcess::new();
        agplatform::TestRequest::new();

        let platform = test_platform()
            .with_process(agplatform::TestProcess::new())
            .with_request(agplatform::TestRequest::new());
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
    fn test_platform_should_run_commands() {
        let platform = test_platform();
        platform.process.set_output(
            "git",
            CommandOutput {
                status: Some(0),
                ..Default::default()
            },
        );

        std::process::Command::new("git")
            .run_blocking(platform.process())
            .unwrap();

        assert_eq!(platform.process.calls(), ["git"]);
    }

    #[test]
    fn test_platform_should_send_requests() {
        let platform = test_platform();
        platform.request.set_response(
            "",
            RequestOutput {
                status: 200,
                ..Default::default()
            },
        );

        platform
            .request()
            .blocking_client()
            .unwrap()
            .get("https://example.com")
            .send_blocking(platform.request())
            .unwrap();

        assert_eq!(platform.request.calls(), ["GET https://example.com/"]);
    }

    #[test]
    fn test_platform_should_resume_interrupted_download() {
        // GIVEN a leader dropping the connection in the middle of a snapshot
        let platform = test_platform();
        let mut headers = HeaderMap::new();
        headers.insert("x-snapshot-id", "7".parse().unwrap());
        platform.request.set_interrupted(
            "GET https://leader/snapshot",
            RequestOutput {
                status: 200,
                headers: headers.clone(),
                ..Default::default()
            },
            [b"first ".to_vec()],
            TransportErrorKind::Other,
        );
        platform.request.set_response_if(
            |call| {
                call.headers
                    .get("range")
                    .is_some_and(|range| range == "bytes=6-")
            },
            RequestOutput {
                status: 206,
                headers,
                body: b"second".to_vec(),
                ..Default::default()
            },
        );
        let mut partial = Vec::new();
        let mut id = None;

        // WHEN downloading it until it succeeds
        let first = download_snapshot(platform.request(), &mut partial, &mut id);
        let second = download_snapshot(platform.request(), &mut partial, &mut id);

        // THEN the second attempt resumes the first one
        assert!(first.is_err());
        assert!(second.is_ok());
        assert_eq!(partial, b"first second");
        let resumed = &platform.request.call_details()[1];
        assert_eq!(resumed.headers["x-snapshot-resume-id"], "7");
    }

    /// Downloads a snapshot into `partial` like agdb_server, resuming the
    /// part received by an attempt that failed with the snapshot `id`.
    fn download_snapshot(
        request: &impl Request,
        partial: &mut Vec<u8>,
        id: &mut Option<String>,
    ) -> agplatform::Result<()> {
        let mut builder = request.blocking_client()?.get("https://leader/snapshot");

        if let Some(id) = id.as_deref() {
            builder = builder
                .header("range", format!("bytes={}-", partial.len()))
                .header("x-snapshot-resume-id", id);
        }

        let result = builder.send_blocking_with_sink(request, &mut |(), chunk: &[u8]| {
            partial.extend_from_slice(chunk);
            Ok(())
        });
        let output = match &result {
            Ok(output) => Some(output),
            Err(error) => error
                .source_as::<TransportError>()
                .and_then(TransportError::output),
        };
        *id = output
            .and_then(|output| output.headers.get("x-snapshot-id"))
            .and_then(|id| id.to_str().ok())
            .map(str::to_string);

        result.map(|_| ())
    }
}
