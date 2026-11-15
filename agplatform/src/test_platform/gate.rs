use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::task::Poll;
use std::task::Waker;

/// [internal]
///
/// Holds the runs of a test double until it is opened, with a failure `F`
/// or without one. Clones share the gate. Waiting blocks only the task, so
/// a blocking run waits on its thread, see
/// [`block_on`](crate::interceptor::block_on).
pub(crate) struct Gate<F>(Arc<Mutex<GateState<F>>>);

struct GateState<F> {
    opened: Option<Option<F>>,
    waiting: Vec<Waker>,
}

impl<F: Clone> Gate<F> {
    pub(crate) fn new() -> Self {
        Self(Arc::new(Mutex::new(GateState {
            opened: None,
            waiting: Vec::new(),
        })))
    }

    /// Lets the held and later runs through with `failure`. Does nothing if
    /// the gate was opened already.
    pub(crate) fn open(&self, failure: Option<F>) {
        let waiting = {
            let mut state = self.state();
            state.opened.get_or_insert(failure);
            std::mem::take(&mut state.waiting)
        };
        waiting.into_iter().for_each(Waker::wake);
    }

    /// Waits until the gate is opened, returning its failure.
    pub(crate) async fn wait(&self) -> Option<F> {
        std::future::poll_fn(|context| {
            let mut state = self.state();

            if let Some(opened) = &state.opened {
                return Poll::Ready(opened.clone());
            }

            if !state
                .waiting
                .iter()
                .any(|waker| waker.will_wake(context.waker()))
            {
                state.waiting.push(context.waker().clone());
            }

            Poll::Pending
        })
        .await
    }

    fn state(&self) -> std::sync::MutexGuard<'_, GateState<F>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl<F> Clone for Gate<F> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// [internal]
///
/// A [`Gate`] held by a test, opened without a failure once the test drops
/// all its clones, e.g. while unwinding from a failed assertion, so that
/// the runs it holds cannot hang the test. Clones share the gate.
pub(crate) struct HeldGate<F: Clone>(Arc<Opener<F>>);

struct Opener<F: Clone>(Gate<F>);

impl<F: Clone> HeldGate<F> {
    pub(crate) fn new() -> Self {
        Self(Arc::new(Opener(Gate::new())))
    }

    pub(crate) fn gate(&self) -> &Gate<F> {
        &self.0.0
    }
}

impl<F: Clone> Clone for HeldGate<F> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<F: Clone> Drop for Opener<F> {
    fn drop(&mut self) {
        self.0.open(None);
    }
}

/// [internal]
///
/// A future pending once before it is ready, e.g. a delay of a test double
/// during which its run can be cancelled.
pub(crate) struct YieldOnce(pub(crate) bool);

impl Future for YieldOnce {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<()> {
        if self.0 {
            std::task::Poll::Ready(())
        } else {
            self.0 = true;
            context.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interceptor::block_on;

    mod gate {
        use super::*;

        mod open {
            use super::*;

            #[test]
            fn open_opened_gate_should_keep_first_failure() {
                let gate = Gate::new();
                gate.open(Some(1));

                gate.open(Some(2));
                gate.open(None);

                assert_eq!(block_on(gate.wait()), Some(1));
            }

            #[test]
            fn open_should_be_seen_by_clones() {
                let gate = Gate::<u8>::new();

                gate.clone().open(None);

                assert!(gate.state().opened.is_some());
            }
        }

        mod wait {
            use super::*;

            #[test]
            fn wait_should_wait_until_opened_by_another_thread() {
                let gate = Gate::new();
                let opener = gate.clone();

                let waiter = std::thread::spawn(move || block_on(gate.wait()));
                std::thread::sleep(std::time::Duration::from_millis(10));
                opener.open(Some(4));

                assert_eq!(waiter.join().unwrap(), Some(4));
            }

            #[test]
            fn wait_opened_gate_should_return_failure_at_once() {
                let gate = Gate::new();
                gate.open(Some(5));

                let mut wait = std::pin::pin!(gate.wait());

                assert_eq!(
                    wait.as_mut()
                        .poll(&mut std::task::Context::from_waker(Waker::noop())),
                    Poll::Ready(Some(5))
                );
            }

            #[test]
            fn wait_polled_again_should_register_waker_once() {
                struct Woken(std::sync::atomic::AtomicBool);

                impl std::task::Wake for Woken {
                    fn wake(self: Arc<Self>) {
                        self.0.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                }

                let gate = Gate::<u8>::new();
                let woken = Arc::new(Woken(std::sync::atomic::AtomicBool::new(false)));
                let waker = Waker::from(woken.clone());
                let mut wait = std::pin::pin!(gate.wait());
                let mut context = std::task::Context::from_waker(&waker);

                assert!(wait.as_mut().poll(&mut context).is_pending());
                assert!(wait.as_mut().poll(&mut context).is_pending());
                assert_eq!(gate.state().waiting.len(), 1);

                gate.open(None);

                assert!(woken.0.load(std::sync::atomic::Ordering::Relaxed));
                assert!(gate.state().waiting.is_empty());
            }
        }
    }

    mod yield_once {
        use super::*;

        mod future {
            use super::*;

            mod poll {
                use super::*;

                #[test]
                fn poll_should_be_pending_once_then_ready() {
                    let mut future = std::pin::pin!(YieldOnce(false));
                    let mut context = std::task::Context::from_waker(Waker::noop());

                    assert!(future.as_mut().poll(&mut context).is_pending());
                    assert!(future.as_mut().poll(&mut context).is_ready());
                }
            }
        }
    }

    mod opener {
        use super::*;

        mod drop {
            use super::*;

            #[test]
            fn drop_should_open_gate_once_all_clones_are_dropped() {
                let held = HeldGate::<()>::new();
                let clone = held.clone();
                let gate = held.gate().clone();

                drop(held);
                let opened_by_first = gate.state().opened.is_some();
                drop(clone);

                assert!(!opened_by_first);
                assert_eq!(block_on(gate.wait()), None);
            }
        }
    }
}
