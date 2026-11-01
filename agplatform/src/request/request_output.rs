use std::borrow::Cow;
use std::time::Duration;
use std::time::SystemTime;

use reqwest::header::HeaderMap;

use crate::Result;
use crate::interceptor::Outcome;

use super::DISPLAY_LINES;

/// Enabled by the `request` feature flag.
///
/// The response to a request.
///
/// Returned from [`Request::send_blocking`](crate::Request::send_blocking) on
/// success and by [`Error::request_output`](crate::Error::request_output) of
/// its error when the status is not a success.
///
/// Outputs are equal when they differ at most in their
/// [`success`](Self::success), which is decided by the run, and their
/// [`display_lines`](Self::display_lines), which only affects how they are
/// displayed. The [default](Self::default) one is an empty `200`, e.g.
/// returned by a [`DryRun`](crate::interceptor::DryRun).
///
/// Its [`Debug`] shows only the length of the body and its start, as the
/// body may be large, e.g. in a `{:?}` of an error.
#[derive(Clone)]
pub struct RequestOutput {
    /// Whether the request succeeded. Set by the run for each attempt to
    /// whether the status is a success (2xx), then may be changed by the
    /// interceptors, see [`HttpRequest`](crate::HttpRequest).
    pub success: bool,

    /// The status code.
    pub status: u16,

    /// The response headers.
    pub headers: HeaderMap,

    /// The whole response body as passed on by the interceptors, empty when
    /// passed to a sink, see
    /// [`Request::send_blocking_with_sink`](crate::Request::send_blocking_with_sink).
    pub body: Vec<u8>,

    /// The wall time from sending the request until the whole body was
    /// received.
    pub duration: Duration,

    /// The number of the last lines of the body shown when displayed, see
    /// [`RequestOptions::display_lines`](crate::RequestOptions::display_lines).
    /// 10 by default.
    pub display_lines: usize,
}

impl RequestOutput {
    /// Returns how long the response asks to wait before a retry with its
    /// `Retry-After` header, as seconds or an HTTP date, if it does, see
    /// [`HttpRequest::retry`](crate::HttpRequest::retry).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::RequestOutput;
    /// use agplatform::reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};
    /// use std::time::Duration;
    ///
    /// let mut headers = HeaderMap::new();
    /// headers.insert(RETRY_AFTER, HeaderValue::from_static("120"));
    /// let output = RequestOutput { status: 503, headers, ..Default::default() };
    ///
    /// assert_eq!(output.retry_after(), Some(Duration::from_secs(120)));
    /// ```
    pub fn retry_after(&self) -> Option<Duration> {
        let value = self
            .headers
            .get(reqwest::header::RETRY_AFTER)?
            .to_str()
            .ok()?
            .trim();

        value.parse().map(Duration::from_secs).ok().or_else(|| {
            http_date(value).map(|date| {
                date.duration_since(SystemTime::now())
                    .unwrap_or(Duration::ZERO)
            })
        })
    }

    /// Returns the output with the status `status`, its body empty, e.g. for
    /// a test double, see
    /// [`TestRequest::set_response`](crate::TestRequest::set_response). Its
    /// success is set by the run from the status.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::RequestOutput;
    ///
    /// let output = RequestOutput::status(404).body("missing").header("content-type", "text/plain");
    /// assert_eq!((output.status, output.text().unwrap()), (404, "missing"));
    /// assert_eq!(output.headers["content-type"], "text/plain");
    /// ```
    pub fn status(status: u16) -> Self {
        Self {
            status,
            ..Default::default()
        }
    }

    /// Returns the output with the body `body`.
    pub fn body(self, body: impl Into<Vec<u8>>) -> Self {
        Self {
            body: body.into(),
            ..self
        }
    }

    /// Returns the output with the header `name` set to `value`, replacing
    /// its previous values.
    ///
    /// # Panics
    ///
    /// Panics if `name` or `value` is not a valid header name or value.
    pub fn header(mut self, name: &str, value: &str) -> Self {
        let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
            .unwrap_or_else(|error| panic!("invalid header name '{name}': {error}"));
        let value = reqwest::header::HeaderValue::from_str(value)
            .unwrap_or_else(|error| panic!("invalid header value '{value}': {error}"));
        self.headers.insert(name, value);
        self
    }

    /// Returns the body as UTF-8. The `charset` of the `Content-Type`
    /// header is not applied, unlike by reqwest's `text()`: a body in
    /// another encoding, e.g. `ISO-8859-1`, is decoded by the caller from
    /// [`Self::body`], e.g. with the `encoding_rs` crate.
    ///
    /// # Errors
    ///
    /// Returns an error if the body is not valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::RequestOutput;
    ///
    /// let output = RequestOutput { body: b"body".to_vec(), ..Default::default() };
    /// assert_eq!(output.text().unwrap(), "body");
    /// ```
    pub fn text(&self) -> Result<&str> {
        std::str::from_utf8(&self.body)
            .map_err(|error| request_error!("body is not valid UTF-8: {error}").with_source(error))
    }

    /// Returns the body as UTF-8, replacing invalid sequences with
    /// `U+FFFD`. The `charset` of the `Content-Type` header is not applied,
    /// see [`Self::text`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::RequestOutput;
    ///
    /// let output = RequestOutput { body: b"body\xFF".to_vec(), ..Default::default() };
    /// assert_eq!(output.text_lossy(), "body\u{FFFD}");
    /// ```
    pub fn text_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }
}

impl Outcome for RequestOutput {
    fn success(&self) -> bool {
        self.success
    }

    fn set_success(&mut self, success: bool) {
        self.success = success;
    }
}

impl std::fmt::Display for RequestOutput {
    /// Shows the status, the duration and the last lines of the body.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "status: {}\nduration: {:?}\nbody:\n{}",
            self.status,
            self.duration,
            crate::utils::Tail {
                text: &self.body,
                lines: self.display_lines,
            }
        )
    }
}

impl std::fmt::Debug for RequestOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let excerpt = &self.body[..self.body.len().min(DEBUG_BODY_BYTES)];
        let omitted = if excerpt.len() < self.body.len() {
            "..."
        } else {
            ""
        };

        f.debug_struct("RequestOutput")
            .field("success", &self.success)
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field(
                "body",
                &format_args!(
                    "{} bytes {:?}{omitted}",
                    self.body.len(),
                    String::from_utf8_lossy(excerpt)
                ),
            )
            .field("duration", &self.duration)
            .field("display_lines", &self.display_lines)
            .finish()
    }
}

impl std::error::Error for RequestOutput {}

impl Default for RequestOutput {
    fn default() -> Self {
        Self {
            success: false,
            status: 200,
            headers: HeaderMap::new(),
            body: Vec::new(),
            duration: Duration::ZERO,
            display_lines: DISPLAY_LINES,
        }
    }
}

impl PartialEq for RequestOutput {
    fn eq(&self, other: &Self) -> bool {
        self.status == other.status
            && self.headers == other.headers
            && self.body == other.body
            && self.duration == other.duration
    }
}

impl Eq for RequestOutput {}

/// The number of the first bytes of the body shown by the [`Debug`] of a
/// [`RequestOutput`].
const DEBUG_BODY_BYTES: usize = 256;

/// Parses an HTTP date in one of its formats (see RFC 9110), the preferred
/// `Sun, 06 Nov 1994 08:49:37 GMT` (IMF-fixdate) or the obsolete
/// `Sunday, 06-Nov-94 08:49:37 GMT` (RFC 850, its year taken from 1970 to
/// 2069) and `Sun Nov  6 08:49:37 1994` (asctime).
fn http_date(value: &str) -> Option<SystemTime> {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let parts = value.split_ascii_whitespace().collect::<Vec<_>>();
    let (day, month, year, time) = match parts[..] {
        [_, day, month, year, time, "GMT"] => (day, month, year.parse::<i64>().ok()?, time),
        [_, date, time, "GMT"] => {
            let mut date = date.split('-');
            let [day, month, year] = std::array::from_fn(|_| date.next().unwrap_or(""));
            let year = year
                .parse::<i64>()
                .ok()
                .filter(|_| year.len() == 2 && date.next().is_none())?;
            (day, month, year + if year < 70 { 2000 } else { 1900 }, time)
        }
        [_, month, day, time, year] => (day, month, year.parse::<i64>().ok()?, time),
        _ => return None,
    };
    let month = MONTHS.iter().position(|name| *name == month)? as i64 + 1;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = [
        31,
        28 + i64::from(leap),
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let day = day
        .parse::<i64>()
        .ok()
        .filter(|day| (1..=days[month as usize - 1]).contains(day))?;
    let mut time = time.split(':');
    let [hour, minute, second] =
        std::array::from_fn(|_| time.next().and_then(|part| part.parse::<i64>().ok()));
    let hour = hour.filter(|hour| (0..24).contains(hour))?;
    let minute = minute.filter(|minute| (0..60).contains(minute))?;
    let second = second.filter(|second| (0..=60).contains(second))?;
    (time.next().is_none() && (0..=9999).contains(&year)).then_some(())?;

    // WHY: the days since the epoch of the civil date, see
    // https://howardhinnant.github.io/date_algorithms.html#days_from_civil
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    let seconds = u64::try_from(days * 86_400 + hour * 3_600 + minute * 60 + second).ok()?;

    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::TransportError;
    use crate::request::TransportErrorKind;
    use crate::request::tests::*;

    mod request_output {
        use super::*;

        mod retry_after {
            use super::*;

            fn asking(retry_after: &str) -> RequestOutput {
                let mut output = output(503, b"");
                output
                    .headers
                    .insert(reqwest::header::RETRY_AFTER, retry_after.parse().unwrap());
                output
            }

            #[test]
            fn retry_after_should_read_seconds_or_http_date() {
                for (value, expected) in [
                    ("5", Some(Duration::from_secs(5))),
                    (" 30 ", Some(Duration::from_secs(30))),
                    ("Sun, 06 Nov 1994 08:49:37 GMT", Some(Duration::ZERO)),
                    ("soon", None),
                    ("\u{e9}", None),
                ] {
                    assert_eq!(asking(value).retry_after(), expected, "value: {value}");
                }
                assert_eq!(output(503, b"").retry_after(), None);
            }

            #[test]
            fn retry_after_future_http_date_should_wait_until_then() {
                let after = asking("Sun, 06 Nov 9994 08:49:37 GMT").retry_after();

                assert!(after.unwrap() > Duration::from_secs(1 << 37));
            }
        }

        mod status {
            use super::*;

            #[test]
            fn status_should_return_output_with_status() {
                let output = RequestOutput::status(503);

                assert_eq!(output.status, 503);
                assert!(output.body.is_empty());
                assert!(!output.success);
            }
        }

        mod body {
            use super::*;

            #[test]
            fn body_should_set_body() {
                assert_eq!(RequestOutput::status(200).body("data").body, b"data");
            }
        }

        mod header {
            use super::*;

            #[test]
            fn header_should_replace_values_of_header() {
                let output = RequestOutput::status(200)
                    .header("etag", "1")
                    .header("etag", "2");

                assert_eq!(
                    output.headers.get_all("etag").iter().collect::<Vec<_>>(),
                    ["2"]
                );
            }

            #[test]
            #[should_panic(expected = "invalid header name 'a b'")]
            fn header_invalid_name_should_panic() {
                let _ = RequestOutput::status(200).header("a b", "1");
            }

            #[test]
            #[should_panic(expected = "invalid header value '\n'")]
            fn header_invalid_value_should_panic() {
                let _ = RequestOutput::status(200).header("etag", "\n");
            }
        }

        mod text {
            use super::*;

            #[test]
            fn text_should_return_utf8_body() {
                assert_eq!(output(200, b"body").text().unwrap(), "body");
            }

            #[test]
            fn text_invalid_utf8_should_return_error() {
                let error = output(200, b"\xFF").text().unwrap_err();

                assert!(error.description().starts_with("body is not valid UTF-8: "));
                assert!(error.source_as::<std::str::Utf8Error>().is_some());
            }
        }

        mod text_lossy {
            use super::*;

            #[test]
            fn text_lossy_should_replace_invalid_sequences() {
                assert_eq!(output(200, b"body\xFF").text_lossy(), "body\u{FFFD}");
            }
        }

        mod outcome {
            use super::*;

            mod success {
                use super::*;

                #[test]
                fn success_should_return_success_field() {
                    let output = RequestOutput {
                        success: true,
                        ..output(500, b"")
                    };

                    assert!(Outcome::success(&output));
                    assert!(!Outcome::success(&RequestOutput::default()));
                }
            }

            mod set_success {
                use super::*;

                #[test]
                fn set_success_should_set_success_field() {
                    let mut output = output(404, b"");

                    output.set_success(true);

                    assert!(output.success);
                }
            }
        }

        mod std_fmt_display {
            use super::*;

            #[test]
            fn fmt_should_show_status_duration_and_last_lines_of_body() {
                let body: String = (1..=12).map(|line| format!("{line}\n")).collect();
                let output = RequestOutput {
                    status: 500,
                    body: body.into_bytes(),
                    duration: Duration::from_millis(5),
                    ..Default::default()
                };

                assert_eq!(
                    output.to_string(),
                    "status: 500\nduration: 5ms\nbody:\n(2 lines omitted)\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n"
                );
                assert_eq!(
                    RequestOutput {
                        display_lines: 1,
                        ..output
                    }
                    .to_string(),
                    "status: 500\nduration: 5ms\nbody:\n(11 lines omitted)\n12\n"
                );
            }
        }

        mod std_fmt_debug {
            use super::*;

            #[test]
            fn fmt_should_show_fields_and_body() {
                assert_eq!(
                    format!("{:?}", output(200, b"body")),
                    "RequestOutput { success: false, status: 200, headers: {}, body: 4 bytes \"body\", \
                     duration: 0ns, display_lines: 10 }"
                );
            }

            #[test]
            fn fmt_large_body_should_show_length_and_start_only() {
                let output = RequestOutput {
                    status: 500,
                    body: vec![b'x'; 100_000],
                    ..Default::default()
                };

                let debug = format!("{output:?}");

                assert!(debug.len() < 500, "{debug}");
                assert!(debug.contains(&format!(
                    "body: 100000 bytes \"{}\"...,",
                    "x".repeat(DEBUG_BODY_BYTES)
                )));
            }

            #[test]
            fn fmt_should_bound_debug_of_errors_holding_large_output() {
                let output = RequestOutput {
                    status: 500,
                    body: vec![b'x'; 100_000],
                    ..Default::default()
                };
                let status = crate::Error::request("status").with_source(output.clone());
                let transport = crate::Error::request("transport").with_source(
                    TransportError::new(TransportErrorKind::Other).with_output(output),
                );

                for error in [status, transport] {
                    let debug = format!("{error:?}");

                    assert!(debug.len() < 1000, "{debug}");
                    assert!(debug.contains("100000 bytes"), "{debug}");
                }
            }
        }

        mod default {
            use super::*;

            #[test]
            fn default_should_be_empty_200_displaying_10_lines() {
                let output = RequestOutput::default();

                assert!(!output.success);
                assert_eq!(output.status, 200);
                assert!(output.headers.is_empty());
                assert!(output.body.is_empty());
                assert_eq!(output.duration, Duration::ZERO);
                assert_eq!(output.display_lines, 10);
            }
        }

        mod partial_eq {
            use super::*;

            mod eq {
                use super::*;

                #[test]
                fn eq_should_ignore_success_and_display_lines() {
                    let output = output(200, b"body");

                    assert_eq!(
                        output,
                        RequestOutput {
                            success: true,
                            display_lines: 1,
                            ..output.clone()
                        }
                    );
                    for other in [
                        RequestOutput {
                            status: 404,
                            ..output.clone()
                        },
                        RequestOutput {
                            headers: HeaderMap::from_iter([(
                                reqwest::header::ETAG,
                                "tag".parse().unwrap(),
                            )]),
                            ..output.clone()
                        },
                        RequestOutput {
                            body: b"other".to_vec(),
                            ..output.clone()
                        },
                        RequestOutput {
                            duration: Duration::from_secs(1),
                            ..output.clone()
                        },
                    ] {
                        assert_ne!(output, other);
                    }
                }
            }
        }
    }

    mod http_date {
        use super::*;

        #[test]
        fn http_date_should_parse_all_formats() {
            for (value, seconds) in [
                ("Sun, 06 Nov 1994 08:49:37 GMT", 784_111_777),
                ("Thu, 01 Jan 1970 00:00:00 GMT", 0),
                ("Tue, 29 Feb 2000 23:59:59 GMT", 951_868_799),
                ("Sunday, 06-Nov-94 08:49:37 GMT", 784_111_777),
                ("Thursday, 01-Jan-70 00:00:00 GMT", 0),
                ("Tuesday, 29-Feb-00 23:59:59 GMT", 951_868_799),
                ("Sun Nov  6 08:49:37 1994", 784_111_777),
                ("Thu Jan 01 00:00:00 1970", 0),
            ] {
                assert_eq!(
                    http_date(value),
                    Some(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)),
                    "value: {value}"
                );
            }
        }

        #[test]
        fn http_date_invalid_should_return_none() {
            for value in [
                "",
                "Sun, 06 Nov 1994 08:49:37 UTC",
                "Sun, 06 Nov 1994 08:49:37 GMT extra",
                "Sun, 06 Foo 1994 08:49:37 GMT",
                "Sun, xx Nov 1994 08:49:37 GMT",
                "Sun, 06 Nov 19x4 08:49:37 GMT",
                "Sun, 06 Nov 1994 xx:49:37 GMT",
                "Sun, 06 Nov 1994 08:xx:37 GMT",
                "Sun, 06 Nov 1994 08:49:xx GMT",
                "Sun, 06 Nov 1994 08 GMT",
                "Sun, 06 Nov 1994 08:49 GMT",
                "Sun, 06 Nov 1994 08:49:37:00 GMT",
                "Wed, 31 Dec 1969 23:59:59 GMT",
                "Sun, 00 Nov 1994 08:49:37 GMT",
                "Sun, 31 Nov 1994 08:49:37 GMT",
                "Sun, 29 Feb 1900 08:49:37 GMT",
                "Sun, 29 Feb 1994 08:49:37 GMT",
                "Sun, 06 Nov 1994 24:49:37 GMT",
                "Sun, 06 Nov 1994 08:60:37 GMT",
                "Sun, 06 Nov 1994 08:49:61 GMT",
                "Sun, 06 Nov 10000 08:49:37 GMT",
                "Sun, 06 Nov 9223372036854775807 08:49:37 GMT",
                "Sun, 06 Nov 1994 9223372036854775807:49:37 GMT",
                "Sunday, 06-Nov-1994 08:49:37 GMT",
                "Sunday, 06-Nov-94-01 08:49:37 GMT",
                "Sunday, 06-Nov-x4 08:49:37 GMT",
                "Sun Nov  6 08:49:37 19x4",
            ] {
                assert_eq!(http_date(value), None, "value: {value}");
            }
        }
    }
}
