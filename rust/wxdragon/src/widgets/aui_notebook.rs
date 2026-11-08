use crate::event::WxEvtHandler;
use crate::prelude::*;
use crate::window::{WindowHandle, WxWidget};
// Window is used by widget_builder macro for backwards compatibility
#[allow(unused_imports)]
use crate::window::Window;
use std::ffi::CString;
use wxdragon_sys as ffi;

// Define style enum for AuiNotebook
widget_style_enum!(
    name: AuiNotebookStyle,
    doc: "Style flags for AuiNotebook.",
    variants: {
        Default: 0x00000001 | 0x00000002 | 0x00000004 | 0x00000010 | 0x00000040 | 0x00000200, "Default AuiNotebook style.",
        Top: 1 << 0, "Tabs along the top (wxAUI_NB_TOP).",
        Bottom: 1 << 3, "Tabs along the bottom (wxAUI_NB_BOTTOM).",
        TabSplit: 1 << 4, "The tab control can be split by dragging a tab (wxAUI_NB_TAB_SPLIT).",
        TabMove: 1 << 5, "A tab can be moved horizontally by dragging (wxAUI_NB_TAB_MOVE).",
        TabExternalMove: 1 << 6, "A tab can be moved to another tab control (wxAUI_NB_TAB_EXTERNAL_MOVE).",
        TabFixedWidth: 1 << 7, "All tabs have the same width (wxAUI_NB_TAB_FIXED_WIDTH).",
        ScrollButtons: 1 << 8, "Left and right scroll buttons; ignored with Multiline (wxAUI_NB_SCROLL_BUTTONS).",
        WindowListButton: 1 << 9, "A drop-down list of all the pages (wxAUI_NB_WINDOWLIST_BUTTON).",
        CloseButton: 1 << 10, "A close button on the tab bar (wxAUI_NB_CLOSE_BUTTON).",
        CloseOnActiveTab: 1 << 11, "The close button on the active tab (wxAUI_NB_CLOSE_ON_ACTIVE_TAB).",
        CloseOnAllTabs: 1 << 12, "The close button on every tab (wxAUI_NB_CLOSE_ON_ALL_TABS).",
        MiddleClickClose: 1 << 13, "Middle click on a tab closes it (wxAUI_NB_MIDDLE_CLICK_CLOSE).",
        Multiline: 1 << 14, "Tabs that do not fit wrap onto more rows instead of scrolling; wxWidgets 3.3 (wxAUI_NB_MULTILINE).",
        PinOnActiveTab: 1 << 15, "The active tab shows a pin to pin or unpin it; wxWidgets 3.3 (wxAUI_NB_PIN_ON_ACTIVE_TAB).",
        UnpinOnAllPinned: 1 << 16, "An unpin button on every pinned tab; wxWidgets 3.3 (wxAUI_NB_UNPIN_ON_ALL_PINNED)."
    },
    default_variant: Default
);

/// Represents a wxAuiNotebook.
///
/// AuiNotebook uses `WindowHandle` internally for safe memory management.
/// When the underlying window is destroyed (by calling `destroy()` or when
/// its parent is destroyed), the handle becomes invalid and all operations
/// become safe no-ops.
///
/// # Example
/// ```ignore
/// let notebook = AuiNotebook::builder(&frame).build();
///
/// // AuiNotebook is Copy - no clone needed for closures!
/// notebook.add_page(&panel, "Tab 1", true);
///
/// // After parent destruction, notebook operations are safe no-ops
/// frame.destroy();
/// assert!(!notebook.is_valid());
/// ```
#[derive(Clone, Copy)]
pub struct AuiNotebook {
    /// Safe handle to the underlying wxAuiNotebook - automatically invalidated on destroy
    handle: WindowHandle,
}

impl AuiNotebook {
    /// Creates a new AuiNotebook from a raw pointer.
    /// This is intended for internal use by the builder.
    fn from_ptr(ptr: *mut ffi::wxd_AuiNotebook_t) -> Self {
        AuiNotebook {
            handle: WindowHandle::new(ptr as *mut ffi::wxd_Window_t),
        }
    }

    /// Creates a new builder for AuiNotebook
    pub fn builder<'a>(parent: &'a dyn WxWidget) -> AuiNotebookBuilder<'a> {
        AuiNotebookBuilder::new(parent)
    }

    /// Helper to get raw notebook pointer, returns null if widget has been destroyed
    #[inline]
    fn notebook_ptr(&self) -> *mut ffi::wxd_AuiNotebook_t {
        self.handle
            .get_ptr()
            .map(|p| p as *mut ffi::wxd_AuiNotebook_t)
            .unwrap_or(std::ptr::null_mut())
    }

    /// Adds a page to the notebook.
    /// No-op if the notebook has been destroyed.
    /// Returns false if the notebook has been destroyed or the operation fails.
    pub fn add_page(&self, page: &impl WxWidget, caption: &str, select: bool) -> bool {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return false;
        }
        let caption_c = CString::new(caption).expect("CString::new failed for caption");
        unsafe {
            // Pass -1 for bitmap_id as a default, assuming no specific bitmap support yet in this wrapper
            ffi::wxd_AuiNotebook_AddPage(ptr, page.handle_ptr(), caption_c.as_ptr(), select, -1)
        }
    }

    /// Returns the number of pages in the notebook.
    /// Returns 0 if the notebook has been destroyed.
    pub fn page_count(&self) -> usize {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return 0;
        }
        unsafe { ffi::wxd_AuiNotebook_GetPageCount(ptr) as usize }
    }

    /// Sets the currently selected page.
    /// Returns 0 if the notebook has been destroyed.
    pub fn set_selection(&self, new_page: usize) -> usize {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return 0;
        }
        unsafe { ffi::wxd_AuiNotebook_SetSelection(ptr, new_page) as usize }
    }

    /// Returns the currently selected page index.
    pub fn get_selection(&self) -> i32 {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return -1;
        }
        unsafe { ffi::wxd_AuiNotebook_GetSelection(ptr) }
    }

    /// Inserts a page into the notebook at the specified position.
    pub fn insert_page(&self, page_idx: usize, page: &impl WxWidget, caption: &str, select: bool) -> bool {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return false;
        }
        let caption_c = CString::new(caption).expect("CString::new failed for caption");
        unsafe { ffi::wxd_AuiNotebook_InsertPage(ptr, page_idx, page.handle_ptr(), caption_c.as_ptr(), select, -1) }
    }

    /// Deletes a page at the given index.
    pub fn delete_page(&self, page_idx: usize) -> bool {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return false;
        }
        unsafe { ffi::wxd_AuiNotebook_DeletePage(ptr, page_idx) }
    }

    /// Removes a page at the given index without destroying the window.
    pub fn remove_page(&self, page_idx: usize) -> bool {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return false;
        }
        unsafe { ffi::wxd_AuiNotebook_RemovePage(ptr, page_idx) }
    }

    /// Deletes all pages.
    pub fn delete_all_pages(&self) -> bool {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return false;
        }
        unsafe { ffi::wxd_AuiNotebook_DeleteAllPages(ptr) }
    }

    /// Returns the window associated with the page at the given index.
    pub fn get_page(&self, page_idx: usize) -> Option<WindowHandle> {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return None;
        }
        let page_ptr = unsafe { ffi::wxd_AuiNotebook_GetPage(ptr, page_idx) };
        if page_ptr.is_null() {
            None
        } else {
            Some(WindowHandle::new(page_ptr))
        }
    }

    /// Returns the index of the specified page window.
    pub fn get_page_index(&self, page: &impl WxWidget) -> i32 {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return -1;
        }
        unsafe { ffi::wxd_AuiNotebook_GetPageIndex(ptr, page.handle_ptr()) }
    }

    /// Returns the tab text for the given page.
    pub fn get_page_text(&self, page_idx: usize) -> String {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return String::new();
        }

        let len = unsafe { ffi::wxd_AuiNotebook_GetPageText(ptr, page_idx, std::ptr::null_mut(), 0) };
        if len < 0 {
            return String::new();
        }

        let mut buf = vec![0u8; len as usize + 1];
        unsafe {
            ffi::wxd_AuiNotebook_GetPageText(ptr, page_idx, buf.as_mut_ptr() as *mut _, buf.len());
        }
        if let Some(last) = buf.last()
            && *last == 0
        {
            buf.pop();
        }
        String::from_utf8_lossy(&buf).to_string()
    }

    /// Sets the tab text for the given page.
    pub fn set_page_text(&self, page_idx: usize, text: &str) -> bool {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return false;
        }
        let text_c = CString::new(text).expect("CString::new failed for text");
        unsafe { ffi::wxd_AuiNotebook_SetPageText(ptr, page_idx, text_c.as_ptr()) }
    }

    /// Returns the underlying WindowHandle for this notebook.
    pub fn window_handle(&self) -> WindowHandle {
        self.handle
    }
}

// Manual WxWidget implementation for AuiNotebook (using WindowHandle)
impl WxWidget for AuiNotebook {
    fn handle_ptr(&self) -> *mut ffi::wxd_Window_t {
        self.handle.get_ptr().unwrap_or(std::ptr::null_mut())
    }

    fn is_valid(&self) -> bool {
        self.handle.is_valid()
    }
}

// Implement WxEvtHandler for event binding
impl WxEvtHandler for AuiNotebook {
    unsafe fn get_event_handler_ptr(&self) -> *mut ffi::wxd_EvtHandler_t {
        self.handle.get_ptr().unwrap_or(std::ptr::null_mut()) as *mut ffi::wxd_EvtHandler_t
    }
}

// Implement common event traits that all Window-based widgets support
impl crate::event::WindowEvents for AuiNotebook {}

impl AuiNotebook {
    /// The tooltip of a page's tab (`wxAuiNotebook::SetPageToolTip`).
    pub fn set_page_tooltip(&self, page_idx: usize, text: &str) -> bool {
        let ptr = self.notebook_ptr();
        if ptr.is_null() {
            return false;
        }
        let c_text = std::ffi::CString::new(text).unwrap_or_default();
        unsafe { ffi::wxd_AuiNotebook_SetPageToolTip(ptr, page_idx, c_text.as_ptr()) }
    }

    /// The tabs' text colours: the other tabs and the current one (the art
    /// provider's `SetColour` / `SetActiveColour`; the flat art, the default
    /// since wxWidgets 3.3, takes the tabs' backgrounds from the system).
    pub fn set_tab_text_colours(&self, normal: crate::color::Colour, active: crate::color::Colour) {
        let ptr = self.notebook_ptr();
        if !ptr.is_null() {
            unsafe { ffi::wxd_AuiNotebook_SetTabTextColours(ptr, normal.into(), active.into()) }
        }
    }

    /// The tabs' font (normal, selected and measuring).
    pub fn set_tab_font(&self, font: &crate::font::Font) {
        let ptr = self.notebook_ptr();
        if !ptr.is_null() {
            unsafe { ffi::wxd_AuiNotebook_SetTabFont(ptr, font.as_ptr()) }
        }
    }

    /// Change the style flags after creation (`SetWindowStyleFlag`).
    pub fn set_notebook_style(&self, style: AuiNotebookStyle) {
        let ptr = self.notebook_ptr();
        if !ptr.is_null() {
            unsafe { ffi::wxd_AuiNotebook_SetWindowStyleFlag(ptr, style.bits()) }
        }
    }
}

/// Events of the notebook's tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuiNotebookEvent {
    /// The selected page changed.
    PageChanged,
    /// The selected page is about to change (can be vetoed).
    PageChanging,
    /// A page is about to be closed by its close button or a middle click
    /// (can be vetoed: "Save changes?").
    PageClose,
    /// A page was closed.
    PageClosed,
    /// Right button released on a tab (a context menu).
    TabRightUp,
    /// Double click on the empty tab bar (a new page).
    BgDclick,
    /// A tab drag ended (the order may have changed).
    DragDone,
}

/// The data of a notebook event.
#[derive(Debug)]
pub struct AuiNotebookEventData {
    pub event: crate::event::Event,
}

impl AuiNotebookEventData {
    pub fn new(event: crate::event::Event) -> Self {
        Self { event }
    }

    /// The page concerned (the new one for a change).
    pub fn get_selection(&self) -> Option<i32> {
        if self.event.is_null() {
            return None;
        }
        // wxAuiNotebookEvent is a wxBookCtrlEvent, as a notebook's.
        let val = unsafe { ffi::wxd_NotebookEvent_GetSelection(self.event.0) };
        if val == ffi::WXD_NOT_FOUND as i32 { None } else { Some(val) }
    }

    /// Stop it (PageChanging, PageClose).
    pub fn veto(&self) {
        self.event.veto();
    }
}

crate::implement_widget_local_event_handlers!(
    AuiNotebook, AuiNotebookEvent, AuiNotebookEventData,
    PageChanged => page_changed, crate::event::EventType::AUINOTEBOOK_PAGE_CHANGED,
    PageChanging => page_changing, crate::event::EventType::AUINOTEBOOK_PAGE_CHANGING,
    PageClose => page_close, crate::event::EventType::AUINOTEBOOK_PAGE_CLOSE,
    PageClosed => page_closed, crate::event::EventType::AUINOTEBOOK_PAGE_CLOSED,
    TabRightUp => tab_right_up, crate::event::EventType::AUINOTEBOOK_TAB_RIGHT_UP,
    BgDclick => bg_dclick, crate::event::EventType::AUINOTEBOOK_BG_DCLICK,
    DragDone => drag_done, crate::event::EventType::AUINOTEBOOK_DRAG_DONE
);


// Use widget_builder macro to create the builder
widget_builder!(
    name: AuiNotebook,
    parent_type: &'a dyn WxWidget,
    style_type: AuiNotebookStyle,
    fields: {},
    build_impl: |slf| {
        let parent_ptr = slf.parent.handle_ptr();
        let ptr = unsafe {
            ffi::wxd_AuiNotebook_Create(
                parent_ptr,
                slf.id,
                slf.pos.into(),
                slf.size.into(),
                slf.style.bits(),
            )
        };
        if ptr.is_null() {
            panic!("Failed to create AuiNotebook");
        }
        // Create a WindowHandle which automatically registers for destroy events
        AuiNotebook::from_ptr(ptr)
    }
);
