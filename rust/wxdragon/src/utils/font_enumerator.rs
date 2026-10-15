//! The fonts installed on the system.

use crate::utils::ArrayString;
use std::ffi::CString;
use wxdragon_sys as ffi;

/// Lists the fonts installed on the system.
pub struct FontEnumerator;

impl FontEnumerator {
    /// The face names of the installed fonts (only the fixed width ones if
    /// `fixed_width_only`). Hidden system fonts (such as macOS'
    /// ".AppleSystemUIFont") are not listed.
    pub fn get_facenames(fixed_width_only: bool) -> Vec<String> {
        let ptr = unsafe { ffi::wxd_FontEnumerator_GetFacenames(fixed_width_only) };
        if ptr.is_null() {
            return Vec::new();
        }
        let names: ArrayString = ptr.into();
        names.get_strings()
    }

    /// Whether a font with this face name is installed. Creating a font with
    /// a misspelled name silently falls back to another face; this tells.
    pub fn is_valid_facename(facename: &str) -> bool {
        let Ok(c_name) = CString::new(facename) else {
            return false;
        };
        unsafe { ffi::wxd_FontEnumerator_IsValidFacename(c_name.as_ptr()) }
    }
}
