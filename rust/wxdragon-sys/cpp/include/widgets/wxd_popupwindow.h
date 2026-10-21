#ifndef WXD_POPUPWINDOW_H
#define WXD_POPUPWINDOW_H

#include "../wxd_types.h"

// --- PopupWindow Functions ---
// A wxPopupWindow: a borderless window without title bar that floats over
// its parent and never takes the keyboard focus. On Wayland it is placed
// relative to its parent window (a wxFrame cannot be placed at all there).
// It does not close by itself: the caller decides when (see wxd_eventfilter.h).
WXD_EXPORTED wxd_PopupWindow_t*
wxd_PopupWindow_Create(wxd_Window_t* parent, wxd_Style_t style);

#endif // WXD_POPUPWINDOW_H
