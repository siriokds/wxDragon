//! A global event filter (wxEventFilter): sees every event the application
//! processes, before any handler.
//!
//! The filter is installed by [`EventFilter::add`] and removed when the
//! returned value is dropped.
//!
//! # Example
//!
//! ```rust,no_run
//! use wxdragon::prelude::*;
//!
//! // Notice every click in the application, whatever window gets it.
//! let filter = EventFilter::add(|event| {
//!     if event.get_event_type() == Some(EventType::LEFT_DOWN) {
//!         println!("click on {:?}", event.get_event_object().map(|w| w.handle_ptr()));
//!     }
//!     FilterResult::Skip
//! });
//! // ... later: `drop(filter)` removes it.
//! ```

use std::ffi::c_void;

use crate::event::Event;
use wxdragon_sys as ffi;

/// What happens to an event after the filter has seen it
/// (wxEventFilter::FilterEvent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterResult {
    /// The event goes on as usual.
    Skip,
    /// The event is not processed and counts as not handled.
    Ignore,
    /// The event is not processed and counts as handled.
    Processed,
}

type Callback = Box<dyn Fn(Event) -> FilterResult>;

/// An installed filter; dropping it removes it.
pub struct EventFilter {
    ptr: *mut ffi::wxd_EventFilter_t,
    // Owned here; the C++ filter only borrows it. Boxed twice so the pointer
    // handed to C++ is thin and stable.
    callback: *mut Callback,
}

impl EventFilter {
    /// Installs `callback`, called with every event before any handler. Must
    /// be called on the main thread, with the application running.
    pub fn add<F>(callback: F) -> EventFilter
    where
        F: Fn(Event) -> FilterResult + 'static,
    {
        let callback: *mut Callback = Box::into_raw(Box::new(Box::new(callback)));
        let ptr = unsafe { ffi::wxd_EventFilter_Add(Some(trampoline), callback as *mut c_void) };
        EventFilter { ptr, callback }
    }
}

impl Drop for EventFilter {
    fn drop(&mut self) {
        unsafe {
            ffi::wxd_EventFilter_Remove(self.ptr);
            drop(Box::from_raw(self.callback));
        }
    }
}

unsafe extern "C" fn trampoline(user_data: *mut c_void, event: *mut ffi::wxd_Event_t) -> i32 {
    if user_data.is_null() {
        return ffi::WXD_EVENT_FILTER_SKIP;
    }
    let callback = unsafe { &*(user_data as *const Callback) };
    let event = unsafe { Event::from_ptr(event) };
    // A panic must not unwind into C++.
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback(event))) {
        Ok(FilterResult::Skip) | Err(_) => ffi::WXD_EVENT_FILTER_SKIP,
        Ok(FilterResult::Ignore) => ffi::WXD_EVENT_FILTER_IGNORE,
        Ok(FilterResult::Processed) => ffi::WXD_EVENT_FILTER_PROCESSED,
    }
}
