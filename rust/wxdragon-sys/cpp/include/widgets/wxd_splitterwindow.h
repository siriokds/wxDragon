#ifndef WXD_SPLITTERWINDOW_H
#define WXD_SPLITTERWINDOW_H

#include "../wxd_types.h"

// --- SplitterWindow Functions ---
WXD_EXPORTED wxd_SplitterWindow_t*
wxd_SplitterWindow_Create(wxd_Window_t* parent, wxd_Id id, wxd_Point pos, wxd_Size size,
                          wxd_Style_t style);

WXD_EXPORTED bool
wxd_SplitterWindow_SplitVertically(wxd_SplitterWindow_t* self, wxd_Window_t* window1,
                                   wxd_Window_t* window2, int sashPosition);

WXD_EXPORTED bool
wxd_SplitterWindow_SplitHorizontally(wxd_SplitterWindow_t* self, wxd_Window_t* window1,
                                     wxd_Window_t* window2, int sashPosition);

WXD_EXPORTED bool
wxd_SplitterWindow_Unsplit(wxd_SplitterWindow_t* self, wxd_Window_t* toRemove);

WXD_EXPORTED void
wxd_SplitterWindow_Initialize(wxd_SplitterWindow_t* self, wxd_Window_t* window);

WXD_EXPORTED void
wxd_SplitterWindow_SetSashPosition(wxd_SplitterWindow_t* self, int position, bool redraw);

WXD_EXPORTED int
wxd_SplitterWindow_GetSashPosition(wxd_SplitterWindow_t* self);

WXD_EXPORTED void
wxd_SplitterWindow_SetMinimumPaneSize(wxd_SplitterWindow_t* self, int paneSize);

WXD_EXPORTED bool
wxd_SplitterWindow_IsSplit(wxd_SplitterWindow_t* self);

// true: the panes are side by side (wxSPLIT_VERTICAL).
WXD_EXPORTED bool
wxd_SplitterWindow_IsSplitVertically(wxd_SplitterWindow_t* self);

WXD_EXPORTED int
wxd_SplitterWindow_GetSashSize(wxd_SplitterWindow_t* self);

WXD_EXPORTED void
wxd_SplitterWindow_SetSashGravity(wxd_SplitterWindow_t* self, double gravity);

// Custom sash drawing: called instead of wxSplitterWindow::DrawSash with the
// sash rectangle and the split direction. `free_user_data` releases
// `user_data` when replaced or when the window is destroyed. Pass a NULL
// callback to restore the default drawing.
typedef void (*wxd_SplitterWindow_DrawSash_Callback)(void* user_data, wxd_DC_t* dc, wxd_Rect sash,
                                                     bool vertical);
typedef void (*wxd_SplitterWindow_FreeUserData_Callback)(void* user_data);

WXD_EXPORTED void
wxd_SplitterWindow_SetDrawSashCallback(wxd_SplitterWindow_t* self,
                                       wxd_SplitterWindow_DrawSash_Callback callback,
                                       wxd_SplitterWindow_FreeUserData_Callback free_user_data,
                                       void* user_data);

// false: no tracker line while dragging (useful with wxSP_LIVE_UPDATE).
WXD_EXPORTED void
wxd_SplitterWindow_SetSashTrackerVisible(wxd_SplitterWindow_t* self, bool visible);

#endif // WXD_SPLITTERWINDOW_H