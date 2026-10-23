//! The main (GUI) thread: the only one allowed to touch wxWidgets.
//!
//! wxWidgets windows may only be used from the thread that runs the event
//! loop. Widgets are not `Send`, so the compiler rejects code that hands one to
//! another thread. A background thread that needs a widget back on the main
//! thread (inside a [`call_after`](crate::call_after) callback, which always
//! runs there) carries it in a [`MainThread`], which travels between threads
//! but only opens on the main one, like glib's `ThreadGuard`.

use wxdragon_sys as ffi;

/// Whether the calling thread is the one wxWidgets runs its windows on.
///
/// Before wxWidgets has started, every thread counts as the main one.
pub fn is_main_thread() -> bool {
    unsafe { ffi::wxd_IsMainThread() }
}

/// A value that may travel to other threads but can only be used on the main
/// thread.
///
/// Widgets are handles (`Copy`), and a handle on another thread does not reach
/// its window. Wrap one to carry it inside a [`call_after`](crate::call_after)
/// callback sent from a background thread, and open it there:
///
/// ```rust,no_run
/// use wxdragon::prelude::*;
///
/// # fn example(label: StaticText) {
/// let label = MainThread::new(label);
/// std::thread::spawn(move || {
///     let text = String::from("done"); // work done off the main thread
///     wxdragon::call_after(Box::new(move || {
///         // The callback runs on the main thread: the label opens there.
///         label.get().set_label(&text);
///     }));
/// });
/// # }
/// ```
///
/// Only `Copy` values: they have no destructor, so dropping one on any thread
/// never touches wxWidgets.
#[derive(Clone, Copy)]
pub struct MainThread<T: Copy>(T);

// The value is only reached through `get`, which checks the thread.
unsafe impl<T: Copy> Send for MainThread<T> {}
unsafe impl<T: Copy> Sync for MainThread<T> {}

impl<T: Copy> MainThread<T> {
    pub fn new(value: T) -> Self {
        MainThread(value)
    }

    /// The value.
    ///
    /// # Panics
    ///
    /// Off the main thread: using a widget there is a bug (the call would not
    /// reach its window).
    #[track_caller]
    pub fn get(&self) -> T {
        assert!(
            is_main_thread(),
            "wxDragon: a MainThread value was opened off the main thread"
        );
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    fn is_send_sync<T: Send + Sync>() {}

    #[test]
    fn a_wrapped_widget_travels_between_threads() {
        is_send_sync::<MainThread<Button>>();
        is_send_sync::<MainThread<Frame>>();
    }
}
