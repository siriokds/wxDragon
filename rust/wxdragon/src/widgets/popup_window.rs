//! Safe wrapper for wxPopupWindow.

use crate::event::{MenuEvents, WindowEvents, WxEvtHandler};
use crate::window::{WindowHandle, WxWidget};
use wxdragon_sys as ffi;

widget_style_enum!(
    name: PopupWindowStyle,
    doc: "Window style flags for PopupWindow",
    variants: {
        BorderNone: ffi::WXD_BORDER_NONE, "No border. (Default)",
        BorderSimple: ffi::WXD_BORDER_SIMPLE, "A simple border."
    },
    default_variant: BorderNone
);

/// A wxPopupWindow: a borderless window without title bar that floats over
/// its parent and never takes the keyboard focus (the keys stay with the
/// parent's frame). It is shown with `show(true)` at the position given by
/// `move_window` (screen coordinates); on Wayland, where a plain window
/// cannot be placed, it is placed relative to its parent.
///
/// It does not close by itself: the caller hides or destroys it (for
/// example on a click elsewhere, seen with an [`EventFilter`](crate::event_filter::EventFilter)).
/// Controls placed inside may not show on every platform (GTK on Wayland):
/// draw the content in a paint handler.
#[derive(Clone, Copy)]
pub struct PopupWindow {
    handle: WindowHandle,
}

impl PopupWindow {
    /// Creates a hidden popup window, child of `parent`.
    pub fn new(parent: &dyn WxWidget, style: PopupWindowStyle) -> PopupWindow {
        let ptr = unsafe { ffi::wxd_PopupWindow_Create(parent.handle_ptr(), style.bits() as ffi::wxd_Style_t) };
        if ptr.is_null() {
            panic!("Failed to create PopupWindow: FFI returned null pointer.");
        }
        PopupWindow {
            handle: WindowHandle::new(ptr as *mut ffi::wxd_Window_t),
        }
    }

    /// Returns the underlying WindowHandle.
    pub fn window_handle(&self) -> WindowHandle {
        self.handle
    }
}

impl WxWidget for PopupWindow {
    fn handle_ptr(&self) -> *mut ffi::wxd_Window_t {
        self.handle.get_ptr().unwrap_or(std::ptr::null_mut())
    }

    fn is_valid(&self) -> bool {
        self.handle.is_valid()
    }
}

impl WxEvtHandler for PopupWindow {
    unsafe fn get_event_handler_ptr(&self) -> *mut ffi::wxd_EvtHandler_t {
        self.handle.get_ptr().unwrap_or(std::ptr::null_mut()) as *mut ffi::wxd_EvtHandler_t
    }
}

impl WindowEvents for PopupWindow {}
impl MenuEvents for PopupWindow {}
