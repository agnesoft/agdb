//! Enabled by the `interceptor` feature flag.
//!
//! The engine running a [`Subject`], e.g. a command or a request, by an
//! [`Executor`] through a chain of [`Interceptor`]s like a middleware, e.g.
//! to retry ([`Retry`]), replace ([`DryRun`], [`Cache`]), accept
//! ([`Accept`]), change or log its runs. Process and request are built on it, and so can be other
//! executors: implement a [`Subject`] giving the types of its output, an
//! [`Executor`] running it, then [`run`](Interceptors::run) (async) or
//! [`run_blocking`](Interceptors::run_blocking) it through its
//! [`Interceptors`].
//!
//! # Examples
//!
//! ```rust
//! use agplatform::Result;
//! use agplatform::interceptor::{
//!     BlockingExecutor, BoxFuture, Captured, Emit, Executor, Interceptors, Outcome, Retry, Run,
//!     Subject,
//! };
//! use std::time::Duration;
//!
//! struct Echo(String);
//!
//! #[derive(Clone)]
//! struct Reply {
//!     code: i32,
//!     text: String,
//! }
//!
//! impl Subject for Echo {
//!     type Output = Reply;
//!     type Stream = ();
//!
//!     fn validate(_run: &Run<Echo>) -> Result {
//!         Ok(())
//!     }
//!
//!     fn replay<'a>(_run: &'a Run<Echo>, output: &'a Reply, emit: &'a mut Emit<'_, Echo>) -> BoxFuture<'a, Result> {
//!         Box::pin(emit.emit((), output.text.as_bytes()))
//!     }
//!
//!     fn finish(_run: &Run<Echo>, output: Reply, mut captured: Captured<Echo>) -> Reply {
//!         let text = String::from_utf8_lossy(&captured.take(())).into_owned();
//!         Reply { text, ..output }
//!     }
//! }
//!
//! impl Outcome for Reply {
//!     fn success(&self) -> bool {
//!         self.code == 0
//!     }
//!
//!     fn set_success(&mut self, success: bool) {
//!         self.code = if success { 0 } else { 1 };
//!     }
//! }
//!
//! struct Shout;
//!
//! impl Executor for Shout {
//!     type Subject = Echo;
//!
//!     fn execute<'a>(&'a self, run: &'a Run<Echo>, mut emit: Emit<'a, Echo>) -> BoxFuture<'a, Result<Reply>> {
//!         let text = run.subject().0.to_uppercase();
//!         Box::pin(async move {
//!             emit.emit((), text.as_bytes()).await?;
//!             Ok(Reply { code: 0, text: String::new() })
//!         })
//!     }
//!
//!     fn sleep(&self, delay: Duration) -> BoxFuture<'static, ()> {
//!         Box::pin(async move { std::thread::sleep(delay) })
//!     }
//! }
//!
//! impl BlockingExecutor for Shout {}
//!
//! let mut interceptors = Interceptors::new();
//! interceptors.push(Retry::new(2, Duration::from_millis(10)));
//! let mut run = Run::new(Echo("hello".to_string()));
//! let output = interceptors.run_blocking(&Shout, &mut run, None);
//!
//! assert_eq!(output.unwrap().text, "HELLO");
//! assert_eq!(run.attempt(), 1);
//!
//! let mut printed = Vec::new();
//! let output = interceptors.run_blocking(
//!     &Shout,
//!     &mut Run::new(Echo("hello".to_string())),
//!     Some(&mut |_: (), data: &[u8]| {
//!         printed.extend_from_slice(data);
//!         Ok(())
//!     }),
//! );
//!
//! assert_eq!(output.unwrap().text, "");
//! assert_eq!(printed, b"HELLO");
//! ```

use std::any::Any;
use std::any::TypeId;
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;
use std::time::Duration;

use crate::Result;

mod accept;
mod cache;
mod dry_run;
mod retry;

pub use accept::Accept;
pub use cache::Cache;
pub use dry_run::{DryRun, DryRunOutput};
pub use retry::{Retry, RetryState};

/// The future returned by [`Interceptor::handle`] and [`Next::attempt`].
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// What a run runs, e.g. a command with its options, giving the types of
/// its output and its rules. The interceptors are for a subject, whichever
/// [`Executor`] runs it, e.g. a real process or a test double.
///
/// The engine applies the rules to each attempt, whether the executor runs
/// it or its replay output is replayed, see [`Run::replay_output`]: it
/// [`validate`](Self::validate)s the run first, then
/// [`finish`](Self::finish)es the output, so that both give the same
/// result for the same output.
///
/// The built-ins rely on two rules an implementation must keep:
///
/// - Replaying a finished output and finishing it again gives the same
///   output: `finish` of what [`replay`](Self::replay) emits for an output
///   is that output, e.g. for [`Cache`] and [`DryRun`]. The replayed data
///   passes through the chunk interceptors again, see
///   [`Interceptor::chunk`].
/// - The output is wholly derived from the executor's output, the run and
///   the captured data: `finish` may recompute the success, so a success
///   set by an interceptor (see [`Outcome::success`]) holds only while that
///   interceptor is in the chain, e.g. for a cached output loaded into
///   another chain.
pub trait Subject: Sized + Send + Sync + 'static {
    /// The output of a run, e.g. of a finished child process.
    type Output: Outcome;

    /// Tells apart the streams of the chunks of the output, e.g. the
    /// standard output and error, `()` when there is one.
    type Stream: Copy + Eq + Send + Sync;

    /// Checks `run` before each attempt.
    ///
    /// # Errors
    ///
    /// Returns the error of a run that cannot be run, e.g. with options
    /// that cannot be combined, which ends the attempt.
    fn validate(run: &Run<Self>) -> Result;

    /// Passes the data of the replay `output` of `run` to `emit` as running
    /// it would, e.g. the standard output and error on their streams.
    ///
    /// # Errors
    ///
    /// Returns the error of `emit`, which ends the attempt.
    fn replay<'a>(
        run: &'a Run<Self>,
        output: &'a Self::Output,
        emit: &'a mut Emit<'_, Self>,
    ) -> BoxFuture<'a, Result>;

    /// Returns the `output` of an attempt of `run`, run or replayed, with
    /// the rules applied, e.g. its success set from its status, and the
    /// data `captured` instead of going to the sink.
    fn finish(run: &Run<Self>, output: Self::Output, captured: Captured<Self>) -> Self::Output;
}

/// The output of a run of a [`Subject`].
pub trait Outcome: Clone + Send + Sync + 'static {
    /// Returns whether the output is a success, as set by
    /// [`Subject::finish`], e.g. from its status, or changed by an
    /// interceptor.
    fn success(&self) -> bool;

    /// Sets whether the output is a success, e.g. by [`Accept`] accepting
    /// a failure.
    fn set_success(&mut self, success: bool);
}

/// A run passing through the interceptors: its subject, e.g. a command,
/// with the state the engine keeps for all subjects.
///
/// The state is kept by the engine for the whole run, across its attempts:
///
/// - [`id`](Self::id) is assigned when the run starts and is the same for
///   all its attempts.
/// - [`attempt`](Self::attempt) counts the times the innermost part of the
///   chain was reached, whether it ran the subject or replayed the output
///   set on the run.
/// - The [`replay_output`](Self::replay_output), set with
///   [`Next::set_replay_output`] (or [`with_replay_output`](Self::with_replay_output)
///   before the run), is replayed on all later attempts
///   instead of running the subject, as if it ran, see
///   [`is_replay`](Self::is_replay). The type of the interceptor setting it
///   is recorded for [`replayed_by`](Self::replayed_by).
/// - [`streamed`](Self::streamed) becomes `true` when a chunk of the output
///   was passed to the sink, after which a retry would pass the same data
///   twice.
/// - [`rejected`](Self::rejected) tells an attempt ended by the engine
///   rather than by the executor.
pub struct Run<S: Subject> {
    subject: S,
    id: u64,
    attempt: u32,
    has_sink: bool,
    streamed: AtomicBool,
    rejected: AtomicBool,
    replay_output: Option<(S::Output, Option<TypeId>)>,
    data: Data,
    state: std::sync::Mutex<Vec<Box<dyn Any + Send>>>,
}

/// Intercepts runs like a middleware, e.g. to change, retry, replace or log
/// them.
///
/// The interceptors of a run form a chain around its [`Executor`], the
/// first one being the outermost. [`handle`](Self::handle) of each one gets
/// the run in a [`Next`], which it may change before passing it on to the
/// rest of the chain with [`Next::attempt`] any number of times, e.g. to
/// retry it, or not at all, and whose result it may change, e.g. to accept
/// a failure. The innermost one is followed by the executor. Each chunk of
/// the output passes through [`chunk`](Self::chunk) of the interceptors as
/// it arrives, from the innermost one outwards, then to the sink of the
/// run, if any, or is captured, see [`Captured`].
///
/// The same [`handle`](Self::handle) serves blocking and async runs: a
/// blocking run waits for its future on the current thread, its executor
/// and [`Next::sleep`] block the thread. An interceptor should therefore
/// await no future needing a runtime, e.g. a timer of tokio, but wait with
/// [`Next::sleep`].
///
/// The replay output set with [`Next::set_replay_output`] is replayed
/// instead of running the subject, see [`Run`] and
/// [`Subject::replay`]. An output returned
/// by an interceptor instead of calling [`Next::attempt`] is taken as it is:
/// it is not replayed, its data reaches neither the sink nor
/// [`chunk`](Self::chunk) and its success is not set by
/// [`Subject::finish`], so the interceptor sets it, see
/// [`Outcome::success`]. An output that is not a success is returned as
/// `Ok`, the caller of [`Interceptors::run`] decides what it means, e.g. an
/// error.
///
/// Each attempt passes the run through the rest of the chain again with the
/// changes made to it before, so an interceptor should replace what it sets
/// rather than add to it.
///
/// Interceptors are told apart by their type, e.g. by
/// [`Run::replayed_by`].
#[allow(unused_variables)]
pub trait Interceptor<S: Subject>: Any + Send + Sync {
    /// Handles the run of `next`, returning its result. Passes it on to the
    /// rest of the chain by default.
    fn handle<'a>(&'a self, next: Next<'a, S>) -> BoxFuture<'a, Result<S::Output>> {
        next.forward()
    }

    /// Handles a chunk of the output of the run as it arrives. Passes it on
    /// by default.
    ///
    /// The data of a replayed output passes through again, e.g. a cached
    /// output whose data was changed by the interceptors when it ran, so a
    /// change should give the same result when made twice, e.g. redacting a
    /// secret rather than adding a prefix.
    ///
    /// # Errors
    ///
    /// An error ends the attempt.
    fn chunk(&self, chunk: Chunk<S>) -> Result {
        chunk.forward()
    }

    /// Called when an async run is cancelled by dropping its future while
    /// its executor runs it or while an interceptor waits with
    /// [`Next::sleep`], on the interceptors running then (all of them, or
    /// those up to the sleeping one) from the innermost one outwards. Not called
    /// when the run panics or is cancelled while an interceptor awaits
    /// another future, which it would have to notice itself, e.g. with a
    /// guard dropped with its future.
    fn cancelled(&self, run: &Run<S>) {}
}

/// The rest of the chain of interceptors of a run, passed to
/// [`Interceptor::handle`] with the run.
pub struct Next<'a, S: Subject> {
    run: &'a mut Run<S>,
    /// The interceptors after the current one, the executor follows them.
    rest: &'a [Arc<dyn Interceptor<S>>],
    executor: &'a mut dyn RunExecutor<S>,
    current: Option<TypeId>,
}

/// A chunk of the output of a run passing through the interceptors, see
/// [`Interceptor::chunk`].
pub struct Chunk<'a, S: Subject> {
    run: &'a Run<S>,
    /// The interceptors the chunk has yet to pass outwards, the sink follows them.
    outer: &'a [Arc<dyn Interceptor<S>>],
    stream: S::Stream,
    data: Cow<'a, [u8]>,
    /// Set when the outermost interceptor passes the chunk on: `None` for
    /// the data unchanged.
    passed: &'a mut Option<Option<Vec<u8>>>,
}

/// The chunks of the output of an attempt that did not go to the sink of
/// its run, as the interceptors passed them on, given to
/// [`Subject::finish`]: all of them for a run without a sink, else those
/// after [`Emit::skip_sink`].
pub struct Captured<S: Subject>(Vec<(S::Stream, Vec<u8>)>);

/// The interceptors of a run, the first one being the outermost.
///
/// Two are equal when they hold the same instances in the same order, e.g.
/// one is a clone of the other.
pub struct Interceptors<S: Subject>(Vec<Arc<dyn Interceptor<S>>>);

/// Receives the chunks of the output of a run that passed through its
/// interceptors, see [`Interceptors::run`], e.g. writing them to a file.
/// Implemented by the closures `FnMut(stream, &[u8]) -> Result`, which
/// block the run while they write. An async sink awaiting in
/// [`write`](Self::write), e.g. sending to a bounded channel, holds the run
/// back until it is done, i.e. applies backpressure.
pub trait Sink<S: Subject>: Send {
    /// Receives the `head` of the output of an attempt before its chunks,
    /// when the executor gives one, see [`Emit::head`], e.g. the status and
    /// headers of a response. Does nothing by default.
    ///
    /// # Errors
    ///
    /// An error ends the attempt, see [`Run::rejected`].
    fn head<'a>(&'a mut self, head: &'a S::Output) -> BoxFuture<'a, Result> {
        let _ = head;
        Box::pin(std::future::ready(Ok(())))
    }

    /// Writes the `data` of `stream`.
    ///
    /// # Errors
    ///
    /// An error ends the attempt, see [`Run::rejected`].
    fn write<'a>(&'a mut self, stream: S::Stream, data: &'a [u8]) -> BoxFuture<'a, Result>;
}

/// Runs a [`Subject`], e.g. by a real process or a test double, blocking or
/// async: a [`BlockingExecutor`] or an [`AsyncExecutor`], or both for one
/// that neither blocks nor needs a runtime, e.g. a test double.
///
/// The engine applies the rules of the subject around
/// [`execute`](Self::execute), see [`Subject`].
pub trait Executor: Sync {
    /// What the executor runs.
    type Subject: Subject;

    /// Runs the subject of a validated `run`, passing its output to `emit`
    /// as it arrives, and returns the output for [`Subject::finish`], which
    /// gets the data from the engine, see [`Captured`].
    ///
    /// # Errors
    ///
    /// Returns the error of a run that failed to run, e.g. of a program not
    /// found, or of `emit`, which should end the attempt.
    fn execute<'a>(
        &'a self,
        run: &'a Run<Self::Subject>,
        emit: Emit<'a, Self::Subject>,
    ) -> BoxFuture<'a, Result<<Self::Subject as Subject>::Output>>;

    /// Waits `delay` between attempts, see [`Next::sleep`], e.g. with the
    /// timer of the runtime of async runs, or not at all for a test double.
    fn sleep(&self, delay: Duration) -> BoxFuture<'static, ()>;
}

/// An [`Executor`] of blocking runs, see [`Interceptors::run_blocking`]:
/// its futures are polled on the current thread without a runtime, so they
/// block it instead of awaiting a future needing one.
pub trait BlockingExecutor: Executor {}

/// An [`Executor`] of async runs, see [`Interceptors::run`]: its futures
/// may need the runtime of the run, e.g. a timer of tokio, and must not
/// block the thread.
pub trait AsyncExecutor: Executor {}

/// Passes the chunks of the output of an attempt through the interceptors
/// to the sink of its run, or captures them, see [`Executor::execute`].
pub struct Emit<'a, S: Subject> {
    run: &'a Run<S>,
    interceptors: &'a [Arc<dyn Interceptor<S>>],
    sink: Option<&'a mut (dyn Sink<S> + 'a)>,
    captured: &'a mut Captured<S>,
}

/// [internal]
///
/// The [`Executor`] of a run bound to it, the innermost part of its chain,
/// hiding the type of the executor from [`Next`] and so from the
/// interceptors.
trait RunExecutor<S: Subject>: Send {
    /// Runs the attempt of `run`.
    fn execute<'a>(&'a mut self, run: &'a mut Run<S>) -> BoxFuture<'a, Result<S::Output>>;

    /// Waits `delay`, see [`Next::sleep`].
    fn sleep(&self, delay: Duration) -> BoxFuture<'static, ()>;

    /// Returns the interceptors of the run.
    fn interceptors(&self) -> &[Arc<dyn Interceptor<S>>];
}

/// [internal]
///
/// The `executor` of a run with its `interceptors`, passing its output to
/// `sink`, if any.
struct BoundExecutor<'a, 's, S: Subject, E: ?Sized> {
    executor: &'a E,
    interceptors: &'a [Arc<dyn Interceptor<S>>],
    sink: Option<&'a mut (dyn Sink<S> + 's)>,
}

/// [internal]
///
/// Tells the interceptors of a run that it is cancelled when dropped before
/// it is disarmed.
struct Cancel<'a, S: Subject> {
    run: &'a Run<S>,
    interceptors: &'a [Arc<dyn Interceptor<S>>],
    armed: bool,
}

/// [internal]
///
/// Wakes the thread blocked in [`block_on`].
struct Unpark(std::thread::Thread);

impl<S: Subject> Run<S> {
    /// Returns the number of the current or last attempt of the run, `1`
    /// from the start of the first one and `0` before.
    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    /// Returns the value of type `T` set on the run, see [`Self::with`].
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.data.get()
    }

    /// Returns `true` if the output of the run goes to a sink, so that it
    /// is not captured, see [`Captured`].
    pub fn has_sink(&self) -> bool {
        self.has_sink
    }

    /// Returns the id of the run, unique within the process among all runs
    /// and the same for all of its attempts, e.g. to tell concurrent runs
    /// apart in an interceptor. `0` until the run starts.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Sets `value` of type `T` on the run, replacing the previous one, e.g.
    /// for the next interceptors.
    pub fn insert<T: Any + Send + Sync>(&mut self, value: T) {
        self.data.insert(value);
    }

    /// Creates the run of `subject`.
    pub fn new(subject: S) -> Self {
        Self {
            subject,
            attempt: 0,
            id: 0,
            has_sink: false,
            streamed: AtomicBool::new(false),
            rejected: AtomicBool::new(false),
            replay_output: None,
            data: Data::default(),
            state: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Returns the output replayed instead of running the subject, see
    /// [`Self::with_replay_output`] and [`Subject::replay`].
    pub fn replay_output(&self) -> Option<&S::Output> {
        self.replay_output.as_ref().map(|(output, _)| output)
    }

    /// Sets the replay output of the run before it runs, like
    /// [`Next::set_replay_output`] but set by no interceptor.
    pub fn with_replay_output(mut self, output: S::Output) -> Self {
        self.replay_output = Some((output, None));
        self
    }

    /// Returns `true` if the run has a replay output, replayed instead of
    /// running its subject, e.g. so that a cache does not store it, see
    /// [`Self::replay_output`].
    pub fn is_replay(&self) -> bool {
        self.replay_output.is_some()
    }

    /// Returns `true` if the last attempt failed because the run is invalid
    /// (see [`Subject::validate`]) or a chunk of its output was not
    /// accepted by an interceptor or the sink, rather than by the executor,
    /// so that retrying it would fail the same way.
    pub fn rejected(&self) -> bool {
        self.rejected.load(Ordering::Relaxed)
    }

    /// Returns `true` if the replay output of the run was set by an
    /// interceptor of type `T`, see [`Next::set_replay_output`].
    pub fn replayed_by<T: Interceptor<S>>(&self) -> bool {
        self.replay_output
            .as_ref()
            .is_some_and(|(_, by)| *by == Some(TypeId::of::<T>()))
    }

    /// Calls `f` with the state of type `T` of the run, created with
    /// [`Default`] on first use, and returns its result. The state lives
    /// for the run, across its attempts, and is cleared when it starts, so
    /// an interceptor keeps what it needs for one run there instead of in
    /// itself, e.g. a line buffer of [`Interceptor::chunk`] flushed by
    /// [`Interceptor::handle`] after the attempt. Unlike the values set
    /// with [`Self::with`], it is changed through `&Run`, e.g. by a
    /// [`Chunk`]. Interceptors keep their own types apart, e.g. a private
    /// type each.
    ///
    /// # Panics
    ///
    /// Panics or deadlocks if `f` calls `state` of the run again.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::interceptor::{Chunk, Interceptor};
    /// use agplatform::HttpRequest;
    ///
    /// #[derive(Default)]
    /// struct Received(usize);
    ///
    /// struct Progress;
    ///
    /// impl Interceptor<HttpRequest> for Progress {
    ///     fn chunk(&self, chunk: Chunk<HttpRequest>) -> agplatform::Result {
    ///         let total = chunk.run().state(|received: &mut Received| {
    ///             received.0 += chunk.data().len();
    ///             received.0
    ///         });
    ///         println!("{total} bytes");
    ///         chunk.forward()
    ///     }
    /// }
    /// ```
    pub fn state<T: Any + Send + Default, R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let index = match state.iter().position(|value| value.is::<T>()) {
            Some(index) => index,
            None => {
                state.push(Box::new(T::default()));
                state.len() - 1
            }
        };

        f(state[index]
            .downcast_mut()
            .expect("the state of the run should be of its type"))
    }

    /// Returns `true` once a chunk of the output of the run was passed to
    /// its sink, even if the sink failed, so that it cannot be retried.
    pub fn streamed(&self) -> bool {
        self.streamed.load(Ordering::Relaxed)
    }

    /// Returns what the run runs, e.g. a command.
    pub fn subject(&self) -> &S {
        &self.subject
    }

    /// Returns what the run runs to be changed, e.g. by an interceptor
    /// before [`Next::attempt`].
    pub fn subject_mut(&mut self) -> &mut S {
        &mut self.subject
    }

    /// Sets `value` of type `T` on the run for the interceptors, replacing
    /// the previous one, e.g. a [`DryRunOutput`] for [`DryRun`]. Read with
    /// [`Self::get`].
    pub fn with<T: Any + Send + Sync>(mut self, value: T) -> Self {
        self.insert(value);
        self
    }
}

impl<'a, S: Subject> Next<'a, S> {
    /// Runs one attempt of the rest of the chain, the next interceptor or
    /// the executor, and returns its result. May be called again, e.g. to
    /// retry the run.
    ///
    /// # Errors
    ///
    /// Returns the error of the rest of the chain, e.g. when the run cannot
    /// be run. An output that is not a success is returned as `Ok`.
    pub fn attempt(&mut self) -> BoxFuture<'_, Result<S::Output>> {
        Next {
            run: &mut *self.run,
            rest: self.rest,
            executor: &mut *self.executor,
            current: self.current,
        }
        .forward()
    }

    /// Passes the run on to the rest of the chain once like
    /// [`Self::attempt`], e.g. as the result of an interceptor that only
    /// changes the run.
    ///
    /// # Errors
    ///
    /// As [`Self::attempt`].
    pub fn forward(self) -> BoxFuture<'a, Result<S::Output>> {
        match self.rest.split_first() {
            Some((next, rest)) => next.handle(Next {
                run: self.run,
                rest,
                executor: self.executor,
                current: Some(type_of(next)),
            }),
            None => self.executor.execute(self.run),
        }
    }

    /// Returns the run.
    pub fn run(&self) -> &Run<S> {
        self.run
    }

    /// Returns the run to be changed, e.g. before [`Self::attempt`].
    pub fn run_mut(&mut self) -> &mut Run<S> {
        self.run
    }

    /// Sets the replay `output` of the run, replayed for its attempts
    /// instead of running the subject,
    /// as if it ran, so its data passes through the interceptors to the
    /// sink. Recorded as set by the interceptor of `self`, see
    /// [`Run::replayed_by`].
    pub fn set_replay_output(&mut self, output: S::Output) {
        self.run.replay_output = Some((output, self.current));
    }

    /// Waits `delay`, e.g. before a retry, unless the executor does not
    /// wait, e.g. that of a test double. Dropping the future while it waits
    /// cancels the run, see [`Interceptor::cancelled`].
    pub fn sleep(&self, delay: Duration) -> BoxFuture<'_, ()> {
        let interceptors = self.executor.interceptors();
        let running = &interceptors[..interceptors.len() - self.rest.len()];
        let (delay, mut cancel) = (self.executor.sleep(delay), Cancel::new(self.run, running));

        Box::pin(async move {
            delay.await;
            cancel.disarm();
        })
    }
}

impl<S: Subject> Chunk<'_, S> {
    /// Returns the data of the chunk.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Passes the chunk on to the next interceptor outwards, after the
    /// outermost one to the sink of the run or captured. A chunk not passed
    /// on is dropped.
    ///
    /// # Errors
    ///
    /// Returns the error of the next interceptor.
    pub fn forward(self) -> Result {
        match self.outer.split_last() {
            Some((next, outer)) => next.chunk(Chunk { outer, ..self }),
            None => {
                *self.passed = Some(match self.data {
                    Cow::Borrowed(_) => None,
                    Cow::Owned(data) => Some(data),
                });
                Ok(())
            }
        }
    }

    /// Returns the run.
    pub fn run(&self) -> &Run<S> {
        self.run
    }

    /// Replaces the data of the chunk passed on, e.g. to redact a secret,
    /// also in the output when it is captured. Empty data drops the chunk
    /// once passed on by the outermost interceptor.
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = Cow::Owned(data);
    }

    /// Returns the stream the chunk is from.
    pub fn stream(&self) -> S::Stream {
        self.stream
    }
}

impl<S: Subject> Interceptors<S> {
    /// Creates an empty list of interceptors.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Inserts `interceptor` at `index`, the outermost at `0`.
    ///
    /// # Panics
    ///
    /// Panics if `index` is greater than [`Self::len`].
    pub fn insert(&mut self, index: usize, interceptor: impl Interceptor<S>) {
        self.0.insert(index, Arc::new(interceptor));
    }

    /// Returns `true` if there are no interceptors.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of the interceptors.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Adds `interceptor` as the innermost one.
    pub fn push(&mut self, interceptor: impl Interceptor<S>) {
        self.0.push(Arc::new(interceptor));
    }

    /// Runs `run` by `executor` through the interceptors, passing its
    /// output to `sink`, if any. The state of `run` is left for the caller,
    /// e.g. its [`attempt`](Run::attempt) count; running it again starts it
    /// anew with what the interceptors set on it.
    ///
    /// # Errors
    ///
    /// Returns the error of the run, e.g. when it cannot be run. An output
    /// that is not a success is returned as `Ok`.
    pub async fn run<E: AsyncExecutor<Subject = S> + ?Sized>(
        &self,
        executor: &E,
        run: &mut Run<S>,
        sink: Option<&mut dyn Sink<S>>,
    ) -> Result<S::Output> {
        self.run_by(executor, run, sink).await
    }

    /// Runs `run` like [`Self::run`], blocking the current thread, see
    /// [`BlockingExecutor`].
    ///
    /// # Errors
    ///
    /// As [`Self::run`].
    pub fn run_blocking<E: BlockingExecutor<Subject = S> + ?Sized>(
        &self,
        executor: &E,
        run: &mut Run<S>,
        sink: Option<&mut dyn Sink<S>>,
    ) -> Result<S::Output> {
        block_on(self.run_by(executor, run, sink))
    }

    async fn run_by<E: Executor<Subject = S> + ?Sized>(
        &self,
        executor: &E,
        run: &mut Run<S>,
        sink: Option<&mut dyn Sink<S>>,
    ) -> Result<S::Output> {
        let interceptors = self.0.as_slice();
        run.id = next_run_id();
        run.attempt = 0;
        run.has_sink = sink.is_some();
        *run.streamed.get_mut() = false;
        run.state
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();

        Next {
            run,
            rest: interceptors,
            executor: &mut BoundExecutor {
                executor,
                interceptors,
                sink,
            },
            current: None,
        }
        .forward()
        .await
    }
}

impl<S: Subject> Clone for Interceptors<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: Subject> Default for Interceptors<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Subject> std::fmt::Debug for Interceptors<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Interceptors({})", self.0.len())
    }
}

impl<S: Subject> PartialEq for Interceptors<S> {
    fn eq(&self, other: &Self) -> bool {
        same_interceptors(&self.0, &other.0)
    }
}

impl<S: Subject> Eq for Interceptors<S> {}

impl<'a, S: Subject> Emit<'a, S> {
    /// Passes the chunk of `data` of `stream` through the interceptors, then
    /// to the sink, or captures it.
    ///
    /// # Errors
    ///
    /// Returns the error of an interceptor or the sink, which ends the
    /// attempt.
    pub async fn emit(&mut self, stream: S::Stream, data: &[u8]) -> Result {
        let result = self.pass(stream, data).await;

        if result.is_err() {
            self.run.rejected.store(true, Ordering::Relaxed);
        }

        result
    }

    /// Passes the `head` of the output of the attempt to the sink of the
    /// run, if it has one and it was not skipped (see
    /// [`skip_sink`](Self::skip_sink)), before the chunks, e.g. the status
    /// and the headers of a response. It does not pass through the
    /// interceptors and does not make the run [`streamed`](Run::streamed),
    /// so a later attempt passes its own head again.
    ///
    /// # Errors
    ///
    /// Returns the error of the sink, which ends the attempt.
    pub async fn head(&mut self, head: &S::Output) -> Result {
        let Some(sink) = self.sink.as_mut() else {
            return Ok(());
        };
        let result = sink.head(head).await;

        if result.is_err() {
            self.run.rejected.store(true, Ordering::Relaxed);
        }

        result
    }

    /// Captures the rest of the chunks of the attempt instead of passing
    /// them to the sink, e.g. the body of a failed response, which can then
    /// still be retried, see [`Run::streamed`].
    pub fn skip_sink(&mut self) {
        self.sink = None;
    }

    /// Removes and returns the data of `stream` captured so far in the
    /// attempt, as the interceptors passed it on, e.g. the part of a
    /// response received before the attempt failed, for its error. The data
    /// taken is not given to [`Subject::finish`].
    pub fn take_captured(&mut self, stream: S::Stream) -> Vec<u8> {
        self.captured.take(stream)
    }

    async fn pass(&mut self, stream: S::Stream, data: &[u8]) -> Result {
        let mut passed = None;
        Chunk {
            run: self.run,
            stream,
            data: Cow::Borrowed(data),
            outer: self.interceptors,
            passed: &mut passed,
        }
        .forward()?;

        let Some(changed) = passed else {
            return Ok(());
        };
        let data = changed.as_deref().unwrap_or(data);

        if data.is_empty() {
            return Ok(());
        }

        match self.sink.as_mut() {
            Some(sink) => {
                self.run.streamed.store(true, Ordering::Relaxed);
                sink.write(stream, data).await
            }
            None => {
                self.captured.push(stream, data);
                Ok(())
            }
        }
    }
}

impl<S: Subject, F: FnMut(S::Stream, &[u8]) -> Result + Send> Sink<S> for F {
    fn write<'a>(&'a mut self, stream: S::Stream, data: &'a [u8]) -> BoxFuture<'a, Result> {
        Box::pin(std::future::ready(self(stream, data)))
    }
}

impl<S: Subject> Captured<S> {
    /// Removes and returns the data of `stream`.
    pub fn take(&mut self, stream: S::Stream) -> Vec<u8> {
        let mut data = Vec::new();
        self.0.retain_mut(|(of, chunk)| {
            let other = *of != stream;
            if !other && data.is_empty() {
                // WHY: moved rather than copied, as there is usually one
                // chunk per stream, e.g. all the output of a process.
                data = std::mem::take(chunk);
            } else if !other {
                data.append(chunk);
            }
            other
        });
        data
    }

    fn push(&mut self, stream: S::Stream, data: &[u8]) {
        match self.0.last_mut() {
            Some((last, chunk)) if *last == stream => chunk.extend_from_slice(data),
            _ => self.0.push((stream, data.to_vec())),
        }
    }
}

impl<S: Subject, E: Executor<Subject = S> + ?Sized> RunExecutor<S> for BoundExecutor<'_, '_, S, E> {
    fn execute<'a>(&'a mut self, run: &'a mut Run<S>) -> BoxFuture<'a, Result<S::Output>> {
        Box::pin(async move {
            run.attempt = run.attempt.saturating_add(1);
            *run.rejected.get_mut() = false;
            let run = &*run;
            let (executor, interceptors) = (self.executor, self.interceptors);
            let mut captured = Captured(Vec::new());
            let mut emit = Emit {
                run,
                interceptors,
                sink: self
                    .sink
                    .as_mut()
                    .map(|sink| &mut **sink as &mut dyn Sink<S>),
                captured: &mut captured,
            };
            let mut cancel = Cancel::new(run, interceptors);
            let result = async {
                S::validate(run).inspect_err(|_| run.rejected.store(true, Ordering::Relaxed))?;

                match run.replay_output() {
                    Some(output) => S::replay(run, output, &mut emit)
                        .await
                        .map(|()| output.clone()),
                    None => executor.execute(run, emit).await,
                }
            }
            .await;
            cancel.disarm();
            result.map(|output| S::finish(run, output, captured))
        })
    }

    fn sleep(&self, delay: Duration) -> BoxFuture<'static, ()> {
        self.executor.sleep(delay)
    }

    fn interceptors(&self) -> &[Arc<dyn Interceptor<S>>] {
        self.interceptors
    }
}

impl<'a, S: Subject> Cancel<'a, S> {
    fn new(run: &'a Run<S>, interceptors: &'a [Arc<dyn Interceptor<S>>]) -> Self {
        Self {
            run,
            interceptors,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl<S: Subject> Drop for Cancel<'_, S> {
    fn drop(&mut self) {
        if self.armed && !std::thread::panicking() {
            self.interceptors
                .iter()
                .rev()
                .for_each(|interceptor| interceptor.cancelled(self.run));
        }
    }
}

impl std::task::Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// Returns the type of the `interceptor` behind its [`Arc`].
///
/// WHY: the [`Any::type_id`] of the [`Arc`] itself would be its own.
fn type_of<S: Subject>(interceptor: &Arc<dyn Interceptor<S>>) -> TypeId {
    let interceptor: &dyn Interceptor<S> = interceptor.as_ref();
    Any::type_id(interceptor)
}

/// [internal]
///
/// Values of distinct types attached to a run, see [`Run::with`].
#[derive(Default)]
pub(crate) struct Data(Vec<Box<dyn Any + Send + Sync>>);

impl Data {
    /// Returns the value of type `T`, if any.
    pub(crate) fn get<T: Any>(&self) -> Option<&T> {
        self.0.iter().find_map(|value| value.downcast_ref())
    }

    /// Sets the value of type `T`, replacing the previous one.
    pub(crate) fn insert<T: Any + Send + Sync>(&mut self, value: T) {
        self.0.retain(|value| !value.is::<T>());
        self.0.push(Box::new(value));
    }
}

/// [internal]
///
/// Returns `true` if the interceptors `a` and `b` are the same instances in
/// the same order.
pub(crate) fn same_interceptors<T: ?Sized>(a: &[Arc<T>], b: &[Arc<T>]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| Arc::ptr_eq(a, b))
}

/// [internal]
///
/// Returns a new id of a run, unique within the process among the runs of
/// all subjects and starting at `1`.
pub(crate) fn next_run_id() -> u64 {
    static NEXT_RUN_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT_RUN_ID.fetch_add(1, Ordering::Relaxed)
}

/// [internal]
///
/// Polls `future` to completion on the current thread, parking it while
/// the future is pending.
pub(crate) fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::park(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::pin::Pin;
    use std::sync::Mutex;

    pub(crate) struct Task {
        pub(crate) name: String,
        pub(crate) invalid: bool,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub(crate) struct Done {
        pub(crate) success: bool,
        pub(crate) status: i32,
        pub(crate) data: Vec<u8>,
        pub(crate) log: Vec<u8>,
        pub(crate) retry_after: Option<Duration>,
    }

    /// Rejects invalid tasks, emits the data (stream 1) and the log (stream
    /// 2) skipping the sink, and sets the success from the status.
    impl Subject for Task {
        type Output = Done;
        type Stream = u8;

        fn validate(run: &Run<Task>) -> Result {
            if run.subject().invalid {
                Err(crate::Error::process(format!(
                    "'{}' is invalid",
                    run.subject().name
                )))
            } else {
                Ok(())
            }
        }

        fn replay<'a>(
            _run: &'a Run<Task>,
            output: &'a Done,
            emit: &'a mut Emit<'_, Task>,
        ) -> BoxFuture<'a, Result> {
            Box::pin(async move {
                emit.emit(1, &output.data).await?;
                emit.skip_sink();
                emit.emit(2, &output.log).await
            })
        }

        fn finish(_run: &Run<Task>, output: Done, mut captured: Captured<Task>) -> Done {
            Done {
                success: output.status == 0,
                data: captured.take(1),
                log: captured.take(2),
                ..output
            }
        }
    }

    impl Outcome for Done {
        fn success(&self) -> bool {
            self.success
        }

        fn set_success(&mut self, success: bool) {
            self.success = success;
        }
    }

    /// Runs the attempts with the `results` in turn, a success when there
    /// are no more, emitting them like [`Task::replay`], after their head
    /// when asked to. Holds the attempts,
    /// or the sleeps, forever when asked to, or the attempts until woken by
    /// another thread, or fails them with the data captured.
    #[derive(Default)]
    pub(crate) struct Script {
        results: Mutex<VecDeque<Result<Done>>>,
        calls: Mutex<Vec<String>>,
        sleeps: Mutex<Vec<Duration>>,
        pub(crate) hold: bool,
        pub(crate) hold_sleep: bool,
        pub(crate) wake_later: bool,
        pub(crate) fail_partial: bool,
        pub(crate) head: bool,
    }

    impl Script {
        pub(crate) fn new(results: impl IntoIterator<Item = Result<Done>>) -> Self {
            Self {
                results: Mutex::new(results.into_iter().collect()),
                ..Default::default()
            }
        }

        pub(crate) fn calls(&self) -> Vec<String> {
            self.calls.lock().unwrap().clone()
        }

        pub(crate) fn sleeps(&self) -> Vec<Duration> {
            self.sleeps.lock().unwrap().clone()
        }
    }

    impl Executor for Script {
        type Subject = Task;

        fn execute<'a>(
            &'a self,
            run: &'a Run<Task>,
            mut emit: Emit<'a, Task>,
        ) -> BoxFuture<'a, Result<Done>> {
            Box::pin(async move {
                self.calls.lock().unwrap().push(run.subject().name.clone());

                if self.hold {
                    std::future::pending::<()>().await;
                }

                if self.wake_later {
                    woken_later().await;
                }

                let result = self.results.lock().unwrap().pop_front();
                let output = result.unwrap_or_else(|| Ok(done(0)))?;

                if self.head {
                    emit.head(&output).await?;
                }

                Task::replay(run, &output, &mut emit).await?;

                if self.fail_partial {
                    return Err(crate::Error::process(format!(
                        "partial {}",
                        emit.take_captured(1).escape_ascii()
                    )));
                }

                Ok(output)
            })
        }

        fn sleep(&self, delay: Duration) -> BoxFuture<'static, ()> {
            self.sleeps.lock().unwrap().push(delay);

            if self.hold_sleep {
                Box::pin(std::future::pending())
            } else {
                Box::pin(std::future::ready(()))
            }
        }
    }

    impl BlockingExecutor for Script {}

    impl AsyncExecutor for Script {}

    /// Completes once woken by another thread, being pending until then.
    fn woken_later() -> impl Future<Output = u32> + Send {
        let woken = Arc::new(AtomicBool::new(false));
        let mut polls = 0;

        std::future::poll_fn(move |context| {
            polls += 1;

            if woken.load(Ordering::SeqCst) {
                return Poll::Ready(polls);
            }

            let (woken, waker) = (woken.clone(), context.waker().clone());
            std::thread::spawn(move || {
                woken.store(true, Ordering::SeqCst);
                waker.wake();
            });
            Poll::Pending
        })
    }

    pub(crate) type Events = Arc<Mutex<Vec<String>>>;

    /// Records the calls of its methods prefixed by its name.
    #[derive(Clone)]
    pub(crate) struct Log {
        name: &'static str,
        events: Events,
    }

    impl Log {
        fn push(&self, event: String) {
            self.events
                .lock()
                .unwrap()
                .push(format!("{} {event}", self.name));
        }
    }

    impl Interceptor<Task> for Log {
        fn handle<'a>(&'a self, mut next: Next<'a, Task>) -> BoxFuture<'a, Result<Done>> {
            Box::pin(async move {
                let result = next.attempt().await;
                self.push(format!(
                    "handled {} {:?}",
                    next.run().attempt(),
                    result.as_ref().ok().map(|output| output.status)
                ));
                result
            })
        }

        fn chunk(&self, chunk: Chunk<Task>) -> Result {
            self.push(format!(
                "chunk {} {}",
                chunk.stream(),
                chunk.data().escape_ascii()
            ));
            chunk.forward()
        }

        fn cancelled(&self, run: &Run<Task>) {
            self.push(format!("cancelled {}", run.subject().name));
        }
    }

    /// An interceptor with the default methods only.
    pub(crate) struct Passing;

    impl Interceptor<Task> for Passing {}

    /// Changes the run before passing it on.
    struct Change(fn(&mut Run<Task>));

    impl Interceptor<Task> for Change {
        fn handle<'a>(&'a self, mut next: Next<'a, Task>) -> BoxFuture<'a, Result<Done>> {
            (self.0)(next.run_mut());
            next.forward()
        }
    }

    /// Records what `look` sees of the run after the rest of the chain.
    struct Inspect {
        look: fn(&Run<Task>) -> String,
        events: Events,
    }

    impl Interceptor<Task> for Inspect {
        fn handle<'a>(&'a self, mut next: Next<'a, Task>) -> BoxFuture<'a, Result<Done>> {
            Box::pin(async move {
                let result = next.attempt().await;
                self.events.lock().unwrap().push((self.look)(next.run()));
                result
            })
        }
    }

    /// Replaces the data of the chunks passed on, or fails them.
    pub(crate) struct Redact(pub(crate) Option<&'static str>);

    impl Interceptor<Task> for Redact {
        fn chunk(&self, mut chunk: Chunk<Task>) -> Result {
            match self.0 {
                Some(data) => {
                    chunk.set_data(data.as_bytes().to_vec());
                    chunk.forward()
                }
                None => Err(crate::Error::process(format!(
                    "rejected {}",
                    chunk.stream()
                ))),
            }
        }
    }

    /// Replaces the data of the chunks passed on by the name of their run.
    struct Name;

    impl Interceptor<Task> for Name {
        fn chunk(&self, mut chunk: Chunk<Task>) -> Result {
            chunk.set_data(chunk.run().subject().name.clone().into_bytes());
            chunk.forward()
        }
    }

    pub(crate) fn log(name: &'static str, events: &Events) -> Log {
        Log {
            name,
            events: events.clone(),
        }
    }

    pub(crate) fn recorded(events: &Events) -> Vec<String> {
        events.lock().unwrap().clone()
    }

    pub(crate) fn task(name: &str) -> Run<Task> {
        Run::new(Task {
            name: name.to_string(),
            invalid: false,
        })
    }

    pub(crate) fn done(status: i32) -> Done {
        Done {
            status,
            ..Default::default()
        }
    }

    pub(crate) fn data(status: i32, data: &str) -> Done {
        Done {
            status,
            data: data.as_bytes().to_vec(),
            ..Default::default()
        }
    }

    pub(crate) fn chain(interceptors: Vec<Arc<dyn Interceptor<Task>>>) -> Interceptors<Task> {
        Interceptors(interceptors)
    }

    pub(crate) fn run(
        script: &Script,
        interceptors: &Interceptors<Task>,
        mut run: Run<Task>,
    ) -> Result<Done> {
        interceptors.run_blocking(script, &mut run, None)
    }

    pub(crate) fn run_to(
        script: &Script,
        interceptors: &Interceptors<Task>,
        mut run: Run<Task>,
        sunk: &mut Vec<(u8, Vec<u8>)>,
    ) -> Result<Done> {
        let mut sink = |stream: u8, data: &[u8]| {
            sunk.push((stream, data.to_vec()));
            Ok(())
        };
        interceptors.run_blocking(script, &mut run, Some(&mut sink))
    }

    fn poll<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
        future.poll(&mut Context::from_waker(Waker::noop()))
    }

    pub(crate) fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    /// Returns `output` replayed for `run` without interceptors and sink,
    /// then finished, as if it ran.
    pub(crate) fn finished<S: Subject>(run: &Run<S>, output: &S::Output) -> S::Output {
        let mut captured = Captured(Vec::new());
        let mut emit = Emit {
            run,
            interceptors: &[],
            sink: None,
            captured: &mut captured,
        };
        block_on(S::replay(run, output, &mut emit)).unwrap();
        S::finish(run, output.clone(), captured)
    }

    /// The contract cases of [`Subject`].
    pub(crate) mod subject {
        use super::*;

        pub(crate) mod replay {
            use super::*;

            /// `output` is one `run` may have when it ran.
            #[cfg_attr(not(any(feature = "process", feature = "request")), allow(dead_code))]
            pub(crate) fn replay_finished_output_should_finish_to_same_output<S: Subject>(
                run: &Run<S>,
                output: &S::Output,
            ) where
                S::Output: PartialEq + std::fmt::Debug,
            {
                let once = finished(run, output);

                assert_eq!(finished(run, &once), once);
            }
        }
    }

    /// The contract cases of [`Interceptor`].
    pub(crate) mod interceptor {
        use super::*;

        pub(crate) mod handle {
            use super::*;

            /// `run` is invalid, see [`Subject::validate`].
            pub(crate) fn handle_invalid_run_should_fail_in_one_attempt<S: Subject>(
                interceptor: Arc<dyn Interceptor<S>>,
                executor: &impl BlockingExecutor<Subject = S>,
                mut run: Run<S>,
            ) {
                let invalid = S::validate(&run).unwrap_err();

                let error = Interceptors(vec![interceptor])
                    .run_blocking(executor, &mut run, None)
                    .err()
                    .unwrap();

                assert_eq!(error.description(), invalid.description());
                assert!(run.rejected());
                assert_eq!(run.attempt(), 1);
            }

            /// `run` has a replay output, see [`Run::with_replay_output`].
            pub(crate) fn handle_replayed_run_should_return_replay_output_once<S: Subject>(
                interceptor: Arc<dyn Interceptor<S>>,
                executor: &impl BlockingExecutor<Subject = S>,
                mut run: Run<S>,
            ) where
                S::Output: PartialEq + std::fmt::Debug,
            {
                let expected = finished(&run, run.replay_output().unwrap());

                let output = Interceptors(vec![interceptor]).run_blocking(executor, &mut run, None);

                assert_eq!(output.ok(), Some(expected));
                assert_eq!(run.attempt(), 1);
            }

            #[test]
            fn handle_should_run_rest_of_chain_by_default() {
                let (script, events) = (Script::new([Ok(data(0, "out"))]), Events::default());
                let interceptors = chain(vec![Arc::new(Passing), Arc::new(log("a", &events))]);

                let output = run(&script, &interceptors, task("x")).unwrap();

                assert_eq!(output.data, b"out");
                assert_eq!(
                    recorded(&events),
                    ["a chunk 1 out", "a chunk 2 ", "a handled 1 Some(0)"]
                );
            }
        }

        pub(crate) mod chunk {
            use super::*;

            pub(crate) fn chunk_should_pass_chunk_on_unchanged<S: Subject>(
                interceptor: Arc<dyn Interceptor<S>>,
                run: &Run<S>,
                stream: S::Stream,
            ) {
                let (outer, mut passed) = ([interceptor], None);

                Chunk {
                    run,
                    outer: &outer,
                    stream,
                    data: Cow::Borrowed(b"data"),
                    passed: &mut passed,
                }
                .forward()
                .unwrap();

                assert_eq!(passed, Some(None));
            }

            #[test]
            fn chunk_should_pass_chunk_on_by_default() {
                let script = Script::new([Ok(data(0, "out"))]);
                let mut sunk = Vec::new();

                run_to(
                    &script,
                    &chain(vec![Arc::new(Passing)]),
                    task("x"),
                    &mut sunk,
                )
                .unwrap();

                assert_eq!(sunk, [(1, b"out".to_vec())]);
            }
        }

        mod cancelled {
            use super::*;

            #[test]
            fn cancelled_should_do_nothing_by_default() {
                let events = Events::default();
                let script = Script {
                    hold: true,
                    ..Default::default()
                };
                let interceptors = chain(vec![Arc::new(log("a", &events)), Arc::new(Passing)]);
                let mut task = task("x");
                let mut future = Box::pin(interceptors.run(&script, &mut task, None));

                assert!(poll(future.as_mut()).is_pending());
                drop(future);

                assert_eq!(recorded(&events), ["a cancelled x"]);
            }
        }
    }

    mod run {
        use super::*;

        mod attempt {
            use super::*;

            #[test]
            fn attempt_should_count_attempts_of_run() {
                let (script, events) = (Script::new([Ok(done(1)), Ok(done(1))]), Events::default());
                let interceptors = chain(vec![
                    Arc::new(Retry::new(2, ms(1))),
                    Arc::new(log("a", &events)),
                ]);

                run(&script, &interceptors, task("x")).unwrap();

                assert_eq!(
                    recorded(&events),
                    [
                        "a chunk 1 ",
                        "a chunk 2 ",
                        "a handled 1 Some(1)",
                        "a chunk 1 ",
                        "a chunk 2 ",
                        "a handled 2 Some(1)",
                        "a chunk 1 ",
                        "a chunk 2 ",
                        "a handled 3 Some(0)",
                    ]
                );
            }

            #[test]
            fn attempt_should_be_zero_before_run() {
                assert_eq!(task("x").attempt(), 0);
            }
        }

        mod get {
            use super::*;

            #[test]
            fn get_missing_value_should_return_none() {
                assert_eq!(task("x").get::<u32>(), None);
            }
        }

        mod has_sink {
            use super::*;

            #[test]
            fn has_sink_should_tell_run_with_sink() {
                let events = Events::default();
                let interceptors = chain(vec![Arc::new(Inspect {
                    look: |run| format!("sink {}", run.has_sink()),
                    events: events.clone(),
                })]);

                run(&Script::default(), &interceptors, task("x")).unwrap();
                run_to(
                    &Script::default(),
                    &interceptors,
                    task("x"),
                    &mut Vec::new(),
                )
                .unwrap();

                assert_eq!(recorded(&events), ["sink false", "sink true"]);
            }
        }

        mod id {
            use super::*;

            #[test]
            fn id_should_be_same_for_attempts_and_unique_per_run() {
                let events = Events::default();
                let interceptors = chain(vec![
                    Arc::new(Retry::new(1, ms(1))),
                    Arc::new(Inspect {
                        look: |run| run.id().to_string(),
                        events: events.clone(),
                    }),
                ]);

                run(&Script::new([Ok(done(1))]), &interceptors, task("x")).unwrap();
                run(&Script::default(), &interceptors, task("y")).unwrap();

                let ids = recorded(&events);
                assert_eq!(ids[0], ids[1]);
                assert_ne!(ids[1], ids[2]);
                assert_ne!(ids[0], "0");
            }

            #[test]
            fn id_should_be_zero_before_run() {
                assert_eq!(task("x").id(), 0);
            }
        }

        mod insert {
            use super::*;

            #[test]
            fn insert_should_replace_value_of_same_type() {
                let mut run = task("x").with(1_u32);

                run.insert(2_u32);

                assert_eq!(run.get::<u32>(), Some(&2));
            }

            #[test]
            fn insert_should_keep_values_of_other_types() {
                let mut run = task("x").with(1_u32).with("a");

                run.insert(2_u32);

                assert_eq!(run.get::<&str>(), Some(&"a"));
                assert_eq!(run.get::<u32>(), Some(&2));
            }
        }

        mod replay_output {
            use super::*;

            #[test]
            fn replay_output_should_return_output_set() {
                assert_eq!(task("x").replay_output(), None);

                let run = task("x").with_replay_output(done(4));

                assert_eq!(run.replay_output(), Some(&done(4)));
            }
        }

        mod with_replay_output {
            use super::*;

            #[test]
            fn with_replay_output_should_replay_output_as_if_it_ran() {
                // GIVEN a replay output set before the chain
                let (script, events) = (Script::default(), Events::default());
                let task = task("x").with_replay_output(Done {
                    log: b"log".to_vec(),
                    ..data(0, "set")
                });
                let mut sunk = Vec::new();

                // WHEN it runs with a sink
                let output = run_to(
                    &script,
                    &chain(vec![Arc::new(log("a", &events))]),
                    task,
                    &mut sunk,
                )
                .unwrap();

                // THEN the subject does not run and the executor replays it
                // AND its data passes through the interceptors to the sink
                assert!(script.calls().is_empty());
                assert!(output.success);
                assert_eq!(output.log, b"log");
                assert_eq!(sunk, [(1, b"set".to_vec())]);
                assert_eq!(
                    recorded(&events),
                    ["a chunk 1 set", "a chunk 2 log", "a handled 1 Some(0)"]
                );
            }
        }

        mod is_replay {
            use super::*;

            #[test]
            fn is_replay_should_tell_run_with_output_set() {
                assert!(!task("x").is_replay());

                let run = task("x").with_replay_output(done(0));

                assert!(run.is_replay());
            }
        }

        mod rejected {
            use super::*;

            #[test]
            fn rejected_should_tell_attempt_ended_by_engine() {
                let events = Events::default();
                let look = |run: &Run<Task>| run.rejected().to_string();
                let interceptors = chain(vec![Arc::new(Inspect {
                    look,
                    events: events.clone(),
                })]);
                let mut invalid = task("x");
                invalid.subject_mut().invalid = true;
                let rejecting = chain(vec![
                    Arc::new(Inspect {
                        look,
                        events: events.clone(),
                    }),
                    Arc::new(Redact(None)),
                ]);

                run(
                    &Script::new([Err(crate::Error::process("io"))]),
                    &interceptors,
                    task("x"),
                )
                .unwrap_err();
                run(&Script::default(), &interceptors, invalid).unwrap_err();
                run(&Script::default(), &rejecting, task("x")).unwrap_err();

                assert_eq!(recorded(&events), ["false", "true", "true"]);
            }
        }

        mod replayed_by {
            use super::*;

            #[test]
            fn replayed_by_should_tell_type_of_interceptor_setting_output() {
                let events = Events::default();
                let interceptors = chain(vec![
                    Arc::new(Inspect {
                        look: |run| {
                            format!(
                                "{} {}",
                                run.replayed_by::<DryRun>(),
                                run.replayed_by::<Passing>()
                            )
                        },
                        events: events.clone(),
                    }),
                    Arc::new(DryRun),
                ]);

                run(&Script::default(), &interceptors, task("x")).unwrap();

                assert_eq!(recorded(&events), ["true false"]);
            }

            #[test]
            fn replayed_by_output_set_outside_chain_should_be_false() {
                let run = task("x").with_replay_output(done(0));

                assert!(!run.replayed_by::<DryRun>());
            }
        }

        mod streamed {
            use super::*;

            #[test]
            fn streamed_should_be_true_once_chunk_passed_to_sink() {
                let events = Events::default();
                let interceptors = chain(vec![Arc::new(Inspect {
                    look: |run| run.streamed().to_string(),
                    events: events.clone(),
                })]);

                run(&Script::new([Ok(data(0, "out"))]), &interceptors, task("x")).unwrap();
                run_to(
                    &Script::new([Ok(data(0, "out"))]),
                    &interceptors,
                    task("x"),
                    &mut Vec::new(),
                )
                .unwrap();

                assert_eq!(recorded(&events), ["false", "true"]);
            }

            #[test]
            fn streamed_chunk_emptied_by_interceptor_should_be_false() {
                let events = Events::default();
                let interceptors = chain(vec![
                    Arc::new(Inspect {
                        look: |run| run.streamed().to_string(),
                        events: events.clone(),
                    }),
                    Arc::new(Redact(Some(""))),
                ]);

                run_to(
                    &Script::new([Ok(data(0, "out"))]),
                    &interceptors,
                    task("x"),
                    &mut Vec::new(),
                )
                .unwrap();

                assert_eq!(recorded(&events), ["false"]);
            }

            #[test]
            fn streamed_should_be_true_when_sink_failed() {
                let events = Events::default();
                let interceptors = chain(vec![Arc::new(Inspect {
                    look: |run| run.streamed().to_string(),
                    events: events.clone(),
                })]);
                let mut sink = |_: u8, _: &[u8]| Err(crate::Error::process("closed"));

                interceptors
                    .run_blocking(
                        &Script::new([Ok(data(0, "out"))]),
                        &mut task("x"),
                        Some(&mut sink),
                    )
                    .unwrap_err();

                assert_eq!(recorded(&events), ["true"]);
            }
        }

        mod state {
            use super::*;

            /// Counts the bytes of the chunks in the state of the run and
            /// records the count after each attempt.
            struct Counting(Events);

            impl Interceptor<Task> for Counting {
                fn handle<'a>(&'a self, mut next: Next<'a, Task>) -> BoxFuture<'a, Result<Done>> {
                    Box::pin(async move {
                        let result = next.attempt().await;
                        let count = next.run().state(|count: &mut usize| *count);
                        self.0.lock().unwrap().push(count.to_string());
                        result
                    })
                }

                fn chunk(&self, chunk: Chunk<Task>) -> Result {
                    chunk
                        .run()
                        .state(|count: &mut usize| *count += chunk.data().len());
                    chunk.forward()
                }
            }

            #[test]
            fn state_should_create_default_and_keep_changes() {
                let run = task("x");

                run.state(|count: &mut u32| *count += 1);
                run.state(|name: &mut String| name.push('a'));
                let count = run.state(|count: &mut u32| {
                    *count += 1;
                    *count
                });

                assert_eq!(count, 2);
                assert_eq!(run.state(|name: &mut String| name.clone()), "a");
            }

            #[test]
            fn state_should_be_shared_by_chunks_and_attempts_of_run() {
                let events = Events::default();
                let interceptors = chain(vec![
                    Arc::new(Retry::new(1, ms(1))),
                    Arc::new(Counting(events.clone())),
                ]);

                run(
                    &Script::new([Ok(data(1, "ab")), Ok(data(0, "cde"))]),
                    &interceptors,
                    task("x"),
                )
                .unwrap();

                assert_eq!(recorded(&events), ["2", "5"]);
            }

            #[test]
            fn state_should_be_cleared_when_run_starts() {
                let events = Events::default();
                let (script, mut run) = (Script::new([Ok(data(0, "ab"))]), task("x"));
                let interceptors = chain(vec![Arc::new(Counting(events.clone()))]);

                for _ in 0..2 {
                    interceptors.run_blocking(&script, &mut run, None).unwrap();
                }

                assert_eq!(recorded(&events), ["2", "0"]);
            }
        }

        mod with {
            use super::*;

            #[test]
            fn with_should_set_value_for_interceptors() {
                let events = Events::default();
                let interceptors = chain(vec![Arc::new(Inspect {
                    look: |run| format!("{:?}", run.get::<u32>()),
                    events: events.clone(),
                })]);

                run(&Script::default(), &interceptors, task("x").with(7_u32)).unwrap();

                assert_eq!(recorded(&events), ["Some(7)"]);
            }
        }
    }

    mod next {
        use super::*;

        mod attempt {
            use super::*;

            #[test]
            fn attempt_should_run_rest_of_chain_each_time() {
                let (script, events) = (Script::new([Ok(done(1)), Ok(done(0))]), Events::default());
                let interceptors = chain(vec![
                    Arc::new(Retry::new(1, ms(1))),
                    Arc::new(log("a", &events)),
                ]);

                run(&script, &interceptors, task("x")).unwrap();

                assert_eq!(script.calls(), ["x", "x"]);
                assert_eq!(
                    recorded(&events)
                        .iter()
                        .filter(|event| event.contains("handled"))
                        .count(),
                    2
                );
            }

            #[test]
            fn attempt_failing_output_should_be_ok() {
                let events = Events::default();
                let interceptors = chain(vec![Arc::new(log("a", &events))]);

                let output = run(&Script::new([Ok(done(1))]), &interceptors, task("x")).unwrap();

                assert!(!output.success);
                assert_eq!(recorded(&events).last().unwrap(), "a handled 1 Some(1)");
            }

            #[test]
            fn attempt_innermost_should_run_subject() {
                let script = Script::new([Ok(data(0, "out"))]);

                let output = run(&script, &Interceptors::new(), task("x")).unwrap();

                assert_eq!(
                    output,
                    Done {
                        success: true,
                        ..data(0, "out")
                    }
                );
                assert_eq!(script.calls(), ["x"]);
            }
        }

        mod forward {
            use super::*;

            #[test]
            fn forward_should_run_rest_of_chain_once() {
                let script = Script::default();
                let interceptors = chain(vec![
                    Arc::new(Change(|run| run.insert(1_u32))),
                    Arc::new(Passing),
                ]);

                run(&script, &interceptors, task("x")).unwrap();

                assert_eq!(script.calls(), ["x"]);
            }
        }

        mod run {
            use super::*;

            #[test]
            fn run_should_return_run_as_changed_before() {
                let events = Events::default();
                let interceptors = chain(vec![
                    Arc::new(Inspect {
                        look: |run| run.subject().name.clone(),
                        events: events.clone(),
                    }),
                    Arc::new(Change(|run| run.subject_mut().name = "changed".to_string())),
                ]);

                run(&Script::default(), &interceptors, task("x")).unwrap();

                assert_eq!(recorded(&events), ["changed"]);
            }
        }

        mod run_mut {
            use super::*;

            #[test]
            fn run_mut_should_change_run_for_rest_of_chain() {
                let script = Script::default();
                let interceptors = chain(vec![Arc::new(Change(|run| {
                    run.subject_mut().name = "changed".to_string()
                }))]);

                run(&script, &interceptors, task("x")).unwrap();

                assert_eq!(script.calls(), ["changed"]);
            }
        }

        mod set_replay_output {
            use super::*;

            #[test]
            fn set_replay_output_should_replay_run_as_set_by_interceptor() {
                let (script, events) = (Script::default(), Events::default());
                let interceptors = chain(vec![
                    Arc::new(Inspect {
                        look: |run| run.replayed_by::<Fixed>().to_string(),
                        events: events.clone(),
                    }),
                    Arc::new(Fixed),
                ]);

                let output = run(&script, &interceptors, task("x")).unwrap();

                assert_eq!(output.data, b"fixed");
                assert!(script.calls().is_empty());
                assert_eq!(recorded(&events), ["true"]);
            }

            /// Sets a fixed output.
            struct Fixed;

            impl Interceptor<Task> for Fixed {
                fn handle<'a>(&'a self, mut next: Next<'a, Task>) -> BoxFuture<'a, Result<Done>> {
                    next.set_replay_output(data(0, "fixed"));
                    next.forward()
                }
            }
        }

        mod sleep {
            use super::*;

            #[test]
            fn sleep_should_wait_by_executor() {
                let script = Script::new([Ok(done(1))]);

                run(
                    &script,
                    &chain(vec![Arc::new(Retry::new(1, ms(5)))]),
                    task("x"),
                )
                .unwrap();

                assert_eq!(script.sleeps(), [ms(5)]);
            }

            #[test]
            fn sleep_cancelled_should_tell_running_interceptors() {
                // GIVEN a retry sleeping forever after a failed attempt
                let events = Events::default();
                let script = Script {
                    hold_sleep: true,
                    ..Script::new([Ok(done(1))])
                };
                let interceptors = chain(vec![
                    Arc::new(log("a", &events)),
                    Arc::new(Retry::new(1, ms(5))),
                    Arc::new(log("b", &events)),
                ]);
                let mut task = task("x");
                let mut future = Box::pin(interceptors.run(&script, &mut task, None));
                assert!(poll(future.as_mut()).is_pending());

                // WHEN the run is cancelled
                drop(future);

                // THEN only the interceptors up to the sleeping one are told
                assert_eq!(
                    recorded(&events)[4..],
                    ["b handled 1 Some(1)", "a cancelled x"]
                );
            }
        }
    }

    mod chunk {
        use super::*;

        #[test]
        fn chunk_should_pass_from_innermost_interceptor_outwards() {
            let events = Events::default();
            let interceptors = chain(vec![
                Arc::new(log("a", &events)),
                Arc::new(log("b", &events)),
            ]);

            run(&Script::new([Ok(data(0, "out"))]), &interceptors, task("x")).unwrap();

            assert_eq!(
                recorded(&events)[..4],
                ["b chunk 1 out", "a chunk 1 out", "b chunk 2 ", "a chunk 2 "]
            );
        }

        mod data {
            use super::*;

            #[test]
            fn data_should_return_data_as_changed_by_inner_interceptors() {
                let events = Events::default();
                let interceptors = chain(vec![
                    Arc::new(log("a", &events)),
                    Arc::new(Redact(Some("***"))),
                ]);

                run(
                    &Script::new([Ok(data(0, "secret"))]),
                    &interceptors,
                    task("x"),
                )
                .unwrap();

                assert_eq!(recorded(&events)[..2], ["a chunk 1 ***", "a chunk 2 ***"]);
            }
        }

        mod forward {
            use super::*;

            #[test]
            fn forward_should_pass_chunk_to_sink() {
                let mut sunk = Vec::new();

                run_to(
                    &Script::new([Ok(data(0, "out"))]),
                    &Interceptors::new(),
                    task("x"),
                    &mut sunk,
                )
                .unwrap();

                assert_eq!(sunk, [(1, b"out".to_vec())]);
            }

            #[test]
            fn forward_failing_interceptor_should_fail_attempt() {
                let error = run(
                    &Script::default(),
                    &chain(vec![Arc::new(Redact(None))]),
                    task("x"),
                )
                .unwrap_err();

                assert_eq!(error.description(), "rejected 1");
            }
        }

        mod run {
            use super::*;

            #[test]
            fn run_should_return_run_of_chunk() {
                let mut sunk = Vec::new();

                run_to(
                    &Script::new([Ok(data(0, "out"))]),
                    &chain(vec![Arc::new(Name)]),
                    task("x"),
                    &mut sunk,
                )
                .unwrap();

                assert_eq!(sunk, [(1, b"x".to_vec())]);
            }
        }

        mod set_data {
            use super::*;

            #[test]
            fn set_data_should_change_data_passed_on_and_captured() {
                let output = Done {
                    log: b"log".to_vec(),
                    ..data(0, "secret")
                };
                let mut sunk = Vec::new();

                let output = run_to(
                    &Script::new([Ok(output)]),
                    &chain(vec![Arc::new(Redact(Some("***")))]),
                    task("x"),
                    &mut sunk,
                )
                .unwrap();

                assert_eq!(sunk, [(1, b"***".to_vec())]);
                assert_eq!(output.log, b"***");
            }

            #[test]
            fn set_data_empty_should_not_pass_chunk_to_sink() {
                let mut sunk = Vec::new();

                run_to(
                    &Script::new([Ok(data(0, "out"))]),
                    &chain(vec![Arc::new(Redact(Some("")))]),
                    task("x"),
                    &mut sunk,
                )
                .unwrap();

                assert!(sunk.is_empty(), "{sunk:?}");
            }
        }

        mod stream {
            use super::*;

            #[test]
            fn stream_should_return_stream_of_chunk() {
                let output = Done {
                    log: b"log".to_vec(),
                    ..data(0, "out")
                };

                let output = run(
                    &Script::new([Ok(output)]),
                    &chain(vec![Arc::new(Only(2))]),
                    task("x"),
                )
                .unwrap();

                assert_eq!((output.data, output.log), (Vec::new(), b"log".to_vec()));
            }

            /// Passes on the chunks of one stream only.
            struct Only(u8);

            impl Interceptor<Task> for Only {
                fn chunk(&self, chunk: Chunk<Task>) -> Result {
                    if chunk.stream() == self.0 {
                        chunk.forward()
                    } else {
                        Ok(())
                    }
                }
            }
        }
    }

    mod interceptors {
        use super::*;

        mod insert {
            use super::*;

            #[test]
            fn insert_should_add_interceptor_at_index() {
                let (script, events) = (Script::default(), Events::default());
                let mut interceptors = chain(vec![Arc::new(log("b", &events))]);

                interceptors.insert(0, log("a", &events));

                run(&script, &interceptors, task("x")).unwrap();
                assert_eq!(
                    recorded(&events)[4..],
                    ["b handled 1 Some(0)", "a handled 1 Some(0)"]
                );
            }

            #[test]
            #[should_panic(expected = "insertion index (is 2) should be <= len (is 1)")]
            fn insert_index_greater_than_len_should_panic() {
                let mut interceptors = chain(vec![Arc::new(Passing)]);

                interceptors.insert(2, Passing);
            }
        }

        mod is_empty {
            use super::*;

            #[test]
            fn is_empty_should_tell_list_without_interceptors() {
                let mut interceptors = Interceptors::<Task>::new();
                assert!(interceptors.is_empty());

                interceptors.push(Passing);

                assert!(!interceptors.is_empty());
            }
        }

        mod len {
            use super::*;

            #[test]
            fn len_should_return_number_of_interceptors() {
                let mut interceptors = Interceptors::<Task>::default();

                interceptors.push(Passing);
                interceptors.push(DryRun);

                assert_eq!(interceptors.len(), 2);
            }
        }

        mod push {
            use super::*;

            #[test]
            fn push_should_add_innermost_interceptor() {
                let (script, events) = (Script::default(), Events::default());
                let mut interceptors = chain(vec![Arc::new(log("a", &events))]);

                interceptors.push(log("b", &events));

                run(&script, &interceptors, task("x")).unwrap();
                assert_eq!(recorded(&events)[..2], ["b chunk 1 ", "a chunk 1 "]);
            }
        }

        #[allow(clippy::module_inception)]
        mod run {
            use super::*;

            #[test]
            fn run_should_be_send() {
                fn send<T: Send>(value: T) -> T {
                    value
                }
                let script = Script::default();
                let interceptors = chain(vec![
                    Arc::new(Retry::new(1, ms(1))),
                    Arc::new(DryRun),
                    Arc::new(Cache::new(|run: &Run<Task>| {
                        Some(run.subject().name.clone())
                    })),
                ]);

                let output =
                    block_on(send(interceptors.run(&script, &mut task("x"), None))).unwrap();

                assert_eq!(
                    output,
                    Done {
                        success: true,
                        ..done(0)
                    }
                );
            }

            #[test]
            fn run_should_leave_state_of_run_for_caller() {
                let script = Script::new([Ok(done(1)), Ok(done(0)), Ok(done(0))]);
                let interceptors = chain(vec![Arc::new(Retry::new(1, ms(1)))]);
                let mut task = task("x");

                interceptors.run_blocking(&script, &mut task, None).unwrap();
                assert_eq!(task.attempt(), 2);
                interceptors.run_blocking(&script, &mut task, None).unwrap();

                assert_eq!(task.attempt(), 1);
            }

            #[test]
            fn run_invalid_run_should_fail_in_chain() {
                let (script, events) = (Script::default(), Events::default());
                let interceptors = chain(vec![
                    Arc::new(log("a", &events)),
                    Arc::new(Change(|run| run.subject_mut().invalid = true)),
                ]);

                let error = run(&script, &interceptors, task("x")).unwrap_err();

                assert_eq!(error.description(), "'x' is invalid");
                assert_eq!(recorded(&events), ["a handled 1 None"]);
                assert!(script.calls().is_empty());
            }

            #[test]
            fn run_failure_should_be_ok() {
                let output =
                    run(&Script::new([Ok(done(2))]), &Interceptors::new(), task("x")).unwrap();

                assert_eq!(output, done(2));
            }

            #[test]
            fn run_should_return_success_changed_by_interceptor() {
                let mut interceptors = Interceptors::new();
                interceptors.push(Accept);

                let output = run(&Script::new([Ok(done(2))]), &interceptors, task("x")).unwrap();

                assert!(output.success);
                assert_eq!(output.status, 2);
            }

            /// Accepts every output.
            struct Accept;

            impl Interceptor<Task> for Accept {
                fn handle<'a>(&'a self, mut next: Next<'a, Task>) -> BoxFuture<'a, Result<Done>> {
                    Box::pin(async move {
                        next.attempt().await.map(|output| Done {
                            success: true,
                            ..output
                        })
                    })
                }
            }
        }

        mod run_blocking {
            use super::*;

            #[test]
            fn run_blocking_should_run_on_current_thread() {
                let script = Script::new([Ok(data(0, "out"))]);

                let output = Interceptors::new()
                    .run_blocking(&script, &mut task("x"), None)
                    .unwrap();

                assert_eq!(output.data, b"out");
            }

            #[test]
            fn run_blocking_should_wait_for_pending_attempt() {
                let script = Script {
                    wake_later: true,
                    ..Script::new([Ok(data(0, "out"))])
                };

                let output = Interceptors::new()
                    .run_blocking(&script, &mut task("x"), None)
                    .unwrap();

                assert_eq!(output.data, b"out");
            }
        }

        mod clone {
            use super::*;

            #[test]
            fn clone_should_share_instances() {
                let mut interceptors = Interceptors::<Task>::new();
                interceptors.push(Passing);

                assert_eq!(interceptors.clone(), interceptors);
            }
        }

        mod std_fmt_debug {
            use super::*;

            #[test]
            fn fmt_should_show_number_of_interceptors() {
                let mut interceptors = Interceptors::<Task>::new();
                interceptors.push(Passing);

                assert_eq!(format!("{interceptors:?}"), "Interceptors(1)");
            }
        }

        mod partial_eq {
            use super::*;

            #[test]
            fn eq_should_compare_instances() {
                let mut a = Interceptors::<Task>::new();
                a.push(Passing);
                let mut b = Interceptors::<Task>::new();
                b.push(Passing);

                assert_ne!(a, b);
                assert_eq!(a, a.clone());
            }

            #[test]
            fn eq_same_instances_in_other_order_should_differ() {
                let (first, second): (Arc<dyn Interceptor<Task>>, Arc<dyn Interceptor<Task>>) =
                    (Arc::new(Passing), Arc::new(Passing));

                assert_ne!(
                    chain(vec![first.clone(), second.clone()]),
                    chain(vec![second, first])
                );
            }
        }
    }

    mod emit {
        use super::*;

        #[allow(clippy::module_inception)]
        mod emit {
            use super::*;

            #[test]
            fn emit_should_pass_through_interceptors_to_sink() {
                let (events, mut sunk) = (Events::default(), Vec::new());

                run_to(
                    &Script::new([Ok(data(0, "out"))]),
                    &chain(vec![Arc::new(log("a", &events))]),
                    task("x"),
                    &mut sunk,
                )
                .unwrap();

                assert_eq!(recorded(&events)[0], "a chunk 1 out");
                assert_eq!(sunk, [(1, b"out".to_vec())]);
            }

            #[test]
            fn emit_failing_sink_should_fail_attempt() {
                let mut sink = |_: u8, _: &[u8]| Err(crate::Error::process("full".to_string()));

                let error = Interceptors::new()
                    .run_blocking(
                        &Script::new([Ok(data(0, "out"))]),
                        &mut task("x"),
                        Some(&mut sink),
                    )
                    .unwrap_err();

                assert_eq!(error.description(), "full");
            }
        }

        mod head {
            use super::*;

            #[test]
            fn head_should_pass_head_to_sink_before_chunks() {
                let script = Script {
                    head: true,
                    ..Script::new([Ok(data(0, "out"))])
                };
                let mut sink = Heads(Vec::new());

                chain(vec![])
                    .run_blocking(&script, &mut task("x"), Some(&mut sink))
                    .unwrap();

                assert_eq!(sink.0, ["head 0", "chunk 1 out"]);
            }

            #[test]
            fn head_without_sink_should_do_nothing() {
                let script = Script {
                    head: true,
                    ..Script::new([Ok(data(0, "out"))])
                };

                let output = run(&script, &chain(vec![]), task("x")).unwrap();

                assert_eq!(output.data, b"out");
            }

            #[test]
            fn head_failing_sink_should_reject_attempt() {
                let script = Script {
                    head: true,
                    ..Script::new([Ok(data(0, "out"))])
                };
                let mut run = task("x");

                let error = chain(vec![Arc::new(Retry::new(1, ms(1)))])
                    .run_blocking(
                        &script,
                        &mut run,
                        Some(&mut Heads(vec!["fail".to_string()])),
                    )
                    .unwrap_err();

                assert_eq!(error.description(), "head refused");
                assert!(run.rejected());
                assert_eq!(script.calls().len(), 1);
            }

            /// Records the heads and the chunks, refusing the head once it
            /// holds `fail`.
            struct Heads(Vec<String>);

            impl Sink<Task> for Heads {
                fn head<'a>(&'a mut self, head: &'a Done) -> BoxFuture<'a, Result> {
                    if self.0.first().is_some_and(|first| first == "fail") {
                        return Box::pin(std::future::ready(Err(crate::Error::process(
                            "head refused".to_string(),
                        ))));
                    }

                    self.0.push(format!("head {}", head.status));
                    Box::pin(std::future::ready(Ok(())))
                }

                fn write<'a>(&'a mut self, stream: u8, data: &'a [u8]) -> BoxFuture<'a, Result> {
                    self.0
                        .push(format!("chunk {stream} {}", String::from_utf8_lossy(data)));
                    Box::pin(std::future::ready(Ok(())))
                }
            }
        }

        mod take_captured {
            use super::*;

            #[test]
            fn take_captured_should_return_data_passed_on_before_failure() {
                // GIVEN an executor failing with the data captured so far
                let script = Script {
                    fail_partial: true,
                    ..Script::new([Ok(data(0, "secret"))])
                };

                // WHEN it runs through an interceptor redacting the chunks
                let error = run(
                    &script,
                    &chain(vec![Arc::new(Redact(Some("***")))]),
                    task("x"),
                )
                .unwrap_err();

                // THEN it gets the data as passed on
                assert_eq!(error.description(), "partial ***");
            }
        }

        mod skip_sink {
            use super::*;

            #[test]
            fn skip_sink_should_capture_rest_of_attempt() {
                let (events, mut sunk) = (Events::default(), Vec::new());
                let output = Done {
                    log: b"log".to_vec(),
                    ..data(1, "out")
                };

                let output = run_to(
                    &Script::new([Ok(output)]),
                    &chain(vec![Arc::new(log("a", &events))]),
                    task("x"),
                    &mut sunk,
                )
                .unwrap();

                assert_eq!(recorded(&events)[1], "a chunk 2 log");
                assert_eq!(sunk, [(1, b"out".to_vec())]);
                assert_eq!((output.data, output.log), (Vec::new(), b"log".to_vec()));
            }
        }
    }

    mod sink {
        use super::*;

        mod head {
            use super::*;

            #[test]
            fn head_should_do_nothing_by_default() {
                let script = Script {
                    head: true,
                    ..Script::new([Ok(data(0, "out"))])
                };
                let mut sunk = Vec::new();

                run_to(&script, &chain(vec![]), task("x"), &mut sunk).unwrap();

                assert_eq!(sunk, [(1, b"out".to_vec())]);
            }
        }

        mod write {
            use super::*;

            #[test]
            fn write_should_be_awaited_for_each_chunk() {
                let mut sink = Later(Vec::new());

                chain(vec![])
                    .run_blocking(
                        &Script::new([Ok(data(0, "out"))]),
                        &mut task("x"),
                        Some(&mut sink),
                    )
                    .unwrap();

                assert_eq!(sink.0, [(1, b"out".to_vec())]);
            }

            /// Records the chunks once woken by another thread.
            struct Later(Vec<(u8, Vec<u8>)>);

            impl Sink<Task> for Later {
                fn write<'a>(&'a mut self, stream: u8, data: &'a [u8]) -> BoxFuture<'a, Result> {
                    Box::pin(async move {
                        woken_later().await;
                        self.0.push((stream, data.to_vec()));
                        Ok(())
                    })
                }
            }
        }
    }

    mod captured {
        use super::*;

        mod take {
            use super::*;

            #[test]
            fn take_should_remove_data_of_stream_in_order() {
                let mut captured = Captured::<Task>(Vec::new());
                captured.push(1, b"a");
                captured.push(1, b"b");
                captured.push(2, b"x");
                captured.push(1, b"c");

                assert_eq!(captured.take(1), b"abc");
                assert_eq!(captured.take(1), b"");
                assert_eq!(captured.take(2), b"x");
            }

            #[test]
            fn take_first_chunk_empty_should_return_later_chunks() {
                let mut captured = Captured::<Task>(Vec::new());
                captured.push(1, b"");
                captured.push(2, b"x");
                captured.push(1, b"a");
                captured.push(2, b"y");
                captured.push(1, b"b");

                assert_eq!(captured.take(1), b"ab");
                assert_eq!(captured.take(2), b"xy");
            }
        }
    }

    mod bound_executor {
        use super::*;

        mod run_executor {
            use super::*;

            mod execute {
                use super::*;

                #[test]
                fn execute_should_apply_rules_of_subject_alike_when_run_or_replayed() {
                    // GIVEN an output with data and a log, failing by its status
                    let output = Done {
                        log: b"log".to_vec(),
                        ..data(1, "out")
                    };
                    let replayed = task("x").with_replay_output(output.clone());
                    let (mut ran_sunk, mut replayed_sunk) = (Vec::new(), Vec::new());

                    // WHEN it is run and replayed with a sink
                    let ran = run_to(
                        &Script::new([Ok(output)]),
                        &Interceptors::new(),
                        task("x"),
                        &mut ran_sunk,
                    )
                    .unwrap();
                    let replayed = run_to(
                        &Script::default(),
                        &Interceptors::new(),
                        replayed,
                        &mut replayed_sunk,
                    )
                    .unwrap();

                    // THEN both are finished alike: not a success, the data
                    // sunk and not captured
                    assert_eq!(ran, replayed);
                    assert_eq!(
                        ran,
                        Done {
                            log: b"log".to_vec(),
                            ..done(1)
                        }
                    );
                    assert_eq!(ran_sunk, replayed_sunk);
                    assert_eq!(ran_sunk, [(1, b"out".to_vec())]);
                }

                #[test]
                fn execute_cancelled_should_tell_interceptors_from_innermost() {
                    let events = Events::default();
                    let script = Script {
                        hold: true,
                        ..Default::default()
                    };
                    let interceptors = chain(vec![
                        Arc::new(log("a", &events)),
                        Arc::new(log("b", &events)),
                    ]);
                    let mut task = task("x");
                    let mut future = Box::pin(interceptors.run(&script, &mut task, None));

                    assert!(poll(future.as_mut()).is_pending());
                    drop(future);

                    assert_eq!(recorded(&events), ["b cancelled x", "a cancelled x"]);
                }
            }
        }
    }

    mod block_on {
        use super::*;

        #[test]
        fn block_on_should_wait_until_woken() {
            let polled = block_on(woken_later());

            assert!(polled >= 2);
        }
    }
}
