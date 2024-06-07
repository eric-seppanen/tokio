//! Hacks on the tokio crate.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static HACKS_ENABLED: AtomicBool = AtomicBool::new(false);

// SAFETY: not really.
static mut LONG_POLL_THRESHOLD: Duration = Duration::MAX;
static mut LONG_POLL_CALLBACK: LongPollCallback = stub_callback;

type GetTypeNameFn = fn() -> &'static str;

/// A callback function, called when a long poll is detected.
pub type LongPollCallback = fn(Duration, &'static str);

fn stub_callback(_: Duration, _: &'static str) {}

/// Enable hacks. These may have serious performance implications.
///
/// `long_poll_threshold` sets the time that a `Future::poll()` is allowed to run.
/// Above this value, we will print a warning.
pub fn init_hacks(long_poll_threshold: Duration, long_poll_callback: LongPollCallback) {
    unsafe {
        LONG_POLL_THRESHOLD = long_poll_threshold;
        LONG_POLL_CALLBACK = long_poll_callback;
    }
    let prev = HACKS_ENABLED.swap(true, Ordering::Release);
    if prev {
        panic!("don't call init_hacks() more than once");
    }
}

/// Print a warning message if the poll time was too long.
pub(crate) fn check_long_poll(poll_duration: Duration, type_fn: GetTypeNameFn) {
    if poll_duration > unsafe { LONG_POLL_THRESHOLD } {
        let name = type_fn();
        let callback_fn = unsafe { LONG_POLL_CALLBACK };
        (callback_fn)(poll_duration, name);
    }
}
