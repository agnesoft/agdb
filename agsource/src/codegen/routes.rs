//! The HTTP routes of a server crate and the check of a client's calls
//! against them: [`SourceCrate::lower_routes`](crate::SourceCrate::lower_routes)
//! reads the axum router the server serves,
//! [`SourceCrate::transport`](crate::SourceCrate::transport) how the
//! implementations of a client's transport trait send their requests with
//! reqwest, and [`check`] matches the request of every call of a client
//! method to its route.

mod axum;
mod check;
mod transport;

pub use check::Checked;
pub use check::check;
pub use transport::Request;
pub use transport::Transport;

use super::lowered::Location;
use super::lowered::Module;
use super::lowered::Ty;

/// A segment of the path of a route.
#[derive(Clone, Debug, PartialEq)]
pub enum Segment {
    Lit(String),
    /// `{name}`: one segment, a path parameter of the handler.
    Param(String),
    /// `{*name}`: the rest of the path.
    Rest(String),
    /// A value only known when the server runs, any number of segments:
    /// its base path.
    Var,
}

/// A route of the router.
#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    /// The method, as the axum `routing` function is named: `get`, `post`.
    pub verb: String,
    /// Without the leading [`Segment::Var`]s, one when the path is only
    /// those.
    pub path: Vec<Segment>,
    /// The path of the handler as written.
    pub handler: String,
    pub at: Location,
    /// What the handler takes and answers, or why that is not known.
    pub io: Result<Io, String>,
}

/// The request and the response of a handler, as serialized.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Io {
    /// The types of the path parameters, in order.
    pub params: Vec<Ty>,
    /// The type of the query string.
    pub query: Option<Ty>,
    pub body: Option<Ty>,
    pub response: Option<Ty>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Routes {
    pub routes: Vec<Route>,
    /// The data types of the server the handlers name.
    pub module: Module,
}

/// The path as written in a route, a [`Segment::Var`] as `{..}`.
pub fn path_text(path: &[Segment]) -> String {
    path.iter()
        .map(|s| match s {
            Segment::Lit(l) => format!("/{l}"),
            Segment::Param(p) => format!("/{{{p}}}"),
            Segment::Rest(r) => format!("/{{*{r}}}"),
            Segment::Var => "/{..}".to_owned(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod path_text {
        use super::*;

        #[test]
        fn path_text_should_write_every_segment() {
            // GIVEN
            let path = [
                Segment::Var,
                Segment::Lit("db".into()),
                Segment::Param("owner".into()),
                Segment::Rest("file".into()),
            ];

            // WHEN
            let text = path_text(&path);

            // THEN
            assert_eq!(text, "/{..}/db/{owner}/{*file}");
        }
    }
}
