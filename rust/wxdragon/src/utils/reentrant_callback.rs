//! Checked access to `FnMut` callbacks invoked across FFI.

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::fmt::Display;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static SKIPPED: AtomicUsize = AtomicUsize::new(0);

/// How many callbacks were skipped because they were called again while still
/// running (a handler that opens a modal dialog, then the same event arrives
/// inside it). Each one is a click, a key or a tick lost: a program's test run
/// can treat any as a failure.
pub fn reentrant_skips() -> usize {
    SKIPPED.load(Ordering::Relaxed)
}

/// Callback data owned by C++, with a temporary strong reference for each dispatch.
/// Releasing the native owner's reference cannot free a callback still on the stack.
/// This protects Rust storage, not the lifetime of the native object itself.
pub(crate) struct FfiCallbackData<T> {
    callbacks: T,
    owner_alive: Cell<bool>,
}

impl<T> FfiCallbackData<T> {
    pub(crate) fn into_raw(callbacks: T) -> *mut c_void {
        Rc::into_raw(Rc::new(Self {
            callbacks,
            owner_alive: Cell::new(true),
        })) as *mut c_void
    }

    /// # Safety
    /// `ptr` must be non-null, come from `into_raw` for the same `T`, still have
    /// a live strong reference, and only be accessed on the callback's thread.
    pub(crate) unsafe fn retain(ptr: *mut c_void) -> Rc<Self> {
        let ptr = ptr as *const Self;
        unsafe {
            Rc::increment_strong_count(ptr);
            Rc::from_raw(ptr)
        }
    }

    /// # Safety
    /// `ptr` must be non-null and come from `into_raw` for the same `T`.
    /// Its native ownership reference must be released exactly once on its thread.
    pub(crate) unsafe fn release(ptr: *mut c_void) {
        let data = unsafe { Rc::from_raw(ptr as *const Self) };
        data.owner_alive.set(false);
    }

    pub(crate) fn is_owner_alive(&self) -> bool {
        self.owner_alive.get()
    }
}

impl<T> Deref for FfiCallbackData<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.callbacks
    }
}

/// Each callback has its own borrow guard: nested calls to other callbacks are allowed.
pub(crate) struct ReentrantCallback<T> {
    callback: RefCell<T>,
}

impl<T> ReentrantCallback<T> {
    pub(crate) fn new(callback: T) -> Self {
        Self {
            callback: RefCell::new(callback),
        }
    }

    /// Reject re-entry before creating a mutable reference to the callback.
    /// The borrow is released on return and on panic; no panic crosses FFI.
    pub(crate) fn call<R>(&self, context: impl Display, fallback: R, invoke: impl FnOnce(&mut T) -> R) -> R {
        super::guard_ffi_callback(&context, None, || {
            let Ok(mut callback) = self.callback.try_borrow_mut() else {
                SKIPPED.fetch_add(1, Ordering::Relaxed);
                log::warn!("Re-entrant invocation of {context} skipped");
                return None;
            };
            Some(invoke(&mut callback))
        })
        .unwrap_or(fallback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A call that arrives while the same callback is running is skipped, and
    /// counted, so a test run can tell that a click or a tick was lost.
    #[test]
    fn a_skipped_reentrant_call_is_counted() {
        let callback = ReentrantCallback::new(0u32);
        let before = reentrant_skips();
        let inner_ran = callback.call("outer", false, |outer_count| {
            *outer_count += 1;
            // Runs: true; skipped: the fallback, false.
            callback.call("inner", false, |_| true)
        });
        assert!(!inner_ran, "the nested call must be skipped");
        // Other tests may skip calls too: at least this one is counted.
        assert!(reentrant_skips() > before);
        // Once the outer call is over, the callback runs again.
        assert!(callback.call("again", false, |_| true));
    }
}
