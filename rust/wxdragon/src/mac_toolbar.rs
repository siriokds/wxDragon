//! A native macOS toolbar for a frame: `NSToolbar` with its own delegate,
//! macOS only.
//!
//! What the toolbar of wxWidgets 3.3 does not give on the Mac:
//! - **stable identifiers**: macOS keeps the user's customization between
//!   runs (wx names its native items after memory addresses);
//! - **menu buttons** (`NSMenuToolbarItem`): a button with a menu and its
//!   small arrow ("16 ⌄"), with or without a primary action ("Open ⌄" with
//!   the recent files);
//! - **groups** (`NSToolbarItemGroup`): items kept together, one unit in the
//!   customization palette;
//! - the **customization palette** and **Show/Hide Toolbar**;
//! - the window's **toolbar style** (`NSWindowToolbarStyle`).
//!
//! A click on an item (or on an entry of its menu) reaches the frame as a
//! `wxEVT_MENU` command with the item's id, as the menu bar's do: one handler
//! serves both. Images are SF Symbols by name. Only public AppKit API.
//!
//! Build the items, set the default set, then [`MacToolbar::install`]. Use
//! this instead of `Frame::create_tool_bar` on the Mac, not with it.

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

use crate::window::{WindowHandle, WxWidget};

unsafe extern "C" {
    fn wxd_MacToolbar_Create(frame: *mut crate::ffi::wxd_Window_t, identifier: *const c_char) -> *mut c_void;
    fn wxd_MacToolbar_AddButton(
        tb: *mut c_void,
        identifier: *const c_char,
        command: c_int,
        label: *const c_char,
        symbol: *const c_char,
        tooltip: *const c_char,
    );
    fn wxd_MacToolbar_AddMenuButton(
        tb: *mut c_void,
        identifier: *const c_char,
        command: c_int,
        label: *const c_char,
        symbol: *const c_char,
        title: *const c_char,
        tooltip: *const c_char,
    );
    fn wxd_MacToolbar_AddGroup(
        tb: *mut c_void,
        identifier: *const c_char,
        label: *const c_char,
        count: c_int,
        members: *const *const c_char,
    );
    fn wxd_MacToolbar_SetDefaultItems(tb: *mut c_void, count: c_int, identifiers: *const *const c_char);
    fn wxd_MacToolbar_Install(tb: *mut c_void, style: c_int, customizable: bool);
    fn wxd_MacToolbar_SetEnabled(tb: *mut c_void, identifier: *const c_char, enabled: bool);
    fn wxd_MacToolbar_SetTitle(tb: *mut c_void, identifier: *const c_char, title: *const c_char);
    fn wxd_MacToolbar_SetMenu(
        tb: *mut c_void,
        identifier: *const c_char,
        count: c_int,
        commands: *const c_int,
        titles: *const *const c_char,
        flags: *const c_int,
    );
    fn wxd_MacToolbar_RunCustomizationPalette(tb: *mut c_void);
    fn wxd_MacToolbar_SetVisible(tb: *mut c_void, visible: bool);
    fn wxd_MacToolbar_IsVisible(tb: *mut c_void) -> bool;
}

/// The window's toolbar style (`NSWindowToolbarStyle`, macOS 11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacToolbarStyle {
    Automatic = 0,
    Expanded = 1,
    Preference = 2,
    Unified = 3,
    /// The toolbar on the title bar's row, with reduced margins.
    UnifiedCompact = 4,
}

/// An entry of a menu button's menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacMenuEntry {
    Item {
        /// The command sent to the frame (`wxEVT_MENU`).
        command: i32,
        title: String,
        checked: bool,
        enabled: bool,
    },
    Separator,
}

/// The standard flexible space, for [`MacToolbar::set_default_items`].
pub const FLEXIBLE_SPACE: &str = "flexible-space";
/// The standard space.
pub const SPACE: &str = "space";

fn c(s: &str) -> CString {
    CString::new(s.replace('\0', "")).unwrap_or_default()
}

fn c_opt(s: Option<&str>) -> Option<CString> {
    s.map(c)
}

fn ptr(s: &Option<CString>) -> *const c_char {
    s.as_ref().map_or(std::ptr::null(), |s| s.as_ptr())
}

/// A frame's native toolbar. Copyable; every call does nothing once the
/// frame is gone.
#[derive(Debug, Clone, Copy)]
pub struct MacToolbar {
    controller: *mut c_void,
    frame: WindowHandle,
}

impl MacToolbar {
    /// A toolbar for `frame`. `identifier` must stay the same between runs
    /// (it names the saved customization), and differ between kinds of
    /// windows.
    pub fn new(frame: &impl WxWidget, identifier: &str) -> Option<MacToolbar> {
        let id = c(identifier);
        // SAFETY: a live frame pointer from wxDragon and a C string.
        let controller = unsafe { wxd_MacToolbar_Create(frame.handle_ptr(), id.as_ptr()) };
        (!controller.is_null()).then(|| MacToolbar { controller, frame: WindowHandle::new(frame.handle_ptr()) })
    }

    fn live(&self) -> Option<*mut c_void> {
        self.frame.is_valid().then_some(self.controller)
    }

    /// A button: an SF Symbol, a label (shown in the customization palette
    /// and under the icon when labels are on), a tooltip; a click sends
    /// `command`.
    pub fn add_button(&self, identifier: &str, command: i32, label: &str, symbol: &str, tooltip: &str) {
        let Some(tb) = self.live() else { return };
        let (i, l, s, t) = (c(identifier), c(label), c(symbol), c(tooltip));
        // SAFETY: the controller lives with the frame's toolbar; C strings.
        unsafe { wxd_MacToolbar_AddButton(tb, i.as_ptr(), command, l.as_ptr(), s.as_ptr(), t.as_ptr()) }
    }

    /// A menu button (`NSMenuToolbarItem`): an SF Symbol or a title (or
    /// both) and the small arrow. With `command`, a click on the button sends
    /// it and the arrow opens the menu; without, a click opens the menu.
    /// The menu is set with [`Self::set_menu`].
    pub fn add_menu_button(
        &self,
        identifier: &str,
        command: Option<i32>,
        label: &str,
        symbol: Option<&str>,
        title: Option<&str>,
        tooltip: &str,
    ) {
        let Some(tb) = self.live() else { return };
        let (i, l, t) = (c(identifier), c(label), c(tooltip));
        let (s, ti) = (c_opt(symbol), c_opt(title));
        // SAFETY: as above; null for a missing symbol or title.
        unsafe {
            wxd_MacToolbar_AddMenuButton(tb, i.as_ptr(), command.unwrap_or(-1), l.as_ptr(), ptr(&s), ptr(&ti), t.as_ptr())
        }
    }

    /// A group (`NSToolbarItemGroup`) of items added before: they stay
    /// together and move as one in the customization palette. Only the group
    /// goes in the default set, not its members.
    pub fn add_group(&self, identifier: &str, label: &str, members: &[&str]) {
        let Some(tb) = self.live() else { return };
        let (i, l) = (c(identifier), c(label));
        let owned: Vec<CString> = members.iter().map(|m| c(m)).collect();
        let raw: Vec<*const c_char> = owned.iter().map(|m| m.as_ptr()).collect();
        // SAFETY: `raw` points into `owned`, alive for the call.
        unsafe { wxd_MacToolbar_AddGroup(tb, i.as_ptr(), l.as_ptr(), raw.len() as c_int, raw.as_ptr()) }
    }

    /// The items shown at first (and by "Use Default Set" in the palette), in
    /// order; [`SPACE`] and [`FLEXIBLE_SPACE`] for the standard spaces.
    pub fn set_default_items(&self, identifiers: &[&str]) {
        let Some(tb) = self.live() else { return };
        let owned: Vec<CString> = identifiers.iter().map(|m| c(m)).collect();
        let raw: Vec<*const c_char> = owned.iter().map(|m| m.as_ptr()).collect();
        // SAFETY: as above.
        unsafe { wxd_MacToolbar_SetDefaultItems(tb, raw.len() as c_int, raw.as_ptr()) }
    }

    /// Puts the toolbar on the window, with `style`. `customizable`: the
    /// user can change it (palette, ⌘-drag) and macOS saves it.
    pub fn install(&self, style: MacToolbarStyle, customizable: bool) {
        let Some(tb) = self.live() else { return };
        // SAFETY: the controller and its frame are alive.
        unsafe { wxd_MacToolbar_Install(tb, style as c_int, customizable) }
    }

    pub fn set_enabled(&self, identifier: &str, enabled: bool) {
        let Some(tb) = self.live() else { return };
        let i = c(identifier);
        // SAFETY: as above.
        unsafe { wxd_MacToolbar_SetEnabled(tb, i.as_ptr(), enabled) }
    }

    /// The text shown in a button ("16").
    pub fn set_title(&self, identifier: &str, title: &str) {
        let Some(tb) = self.live() else { return };
        let (i, t) = (c(identifier), c(title));
        // SAFETY: as above.
        unsafe { wxd_MacToolbar_SetTitle(tb, i.as_ptr(), t.as_ptr()) }
    }

    /// The menu of a menu button.
    pub fn set_menu(&self, identifier: &str, entries: &[MacMenuEntry]) {
        let Some(tb) = self.live() else { return };
        let i = c(identifier);
        let mut commands = Vec::with_capacity(entries.len());
        let mut titles = Vec::with_capacity(entries.len());
        let mut flags = Vec::with_capacity(entries.len());
        for e in entries {
            match e {
                MacMenuEntry::Item { command, title, checked, enabled } => {
                    commands.push(*command as c_int);
                    titles.push(c(title));
                    flags.push(c_int::from(*checked) | if *enabled { 0 } else { 2 });
                }
                MacMenuEntry::Separator => {
                    commands.push(-1);
                    titles.push(c(""));
                    flags.push(4);
                }
            }
        }
        let raw: Vec<*const c_char> = titles.iter().map(|t| t.as_ptr()).collect();
        // SAFETY: three arrays of the same length, alive for the call.
        unsafe {
            wxd_MacToolbar_SetMenu(tb, i.as_ptr(), raw.len() as c_int, commands.as_ptr(), raw.as_ptr(), flags.as_ptr())
        }
    }

    /// The system's "Customize Toolbar…" sheet.
    pub fn run_customization_palette(&self) {
        let Some(tb) = self.live() else { return };
        // SAFETY: as above.
        unsafe { wxd_MacToolbar_RunCustomizationPalette(tb) }
    }

    /// Show or hide the toolbar ("Show Toolbar", ⌥⌘T in macOS apps).
    pub fn set_visible(&self, visible: bool) {
        let Some(tb) = self.live() else { return };
        // SAFETY: as above.
        unsafe { wxd_MacToolbar_SetVisible(tb, visible) }
    }

    pub fn is_visible(&self) -> bool {
        // SAFETY: as above.
        self.live().is_some_and(|tb| unsafe { wxd_MacToolbar_IsVisible(tb) })
    }
}
