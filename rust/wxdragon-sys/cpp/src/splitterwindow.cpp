#include <wx/wxprec.h>
#include <wx/wx.h>
#include "../include/wxdragon.h"
#include <wx/splitter.h>

// A wxSplitterWindow whose sash drawing can be provided by the caller. Without
// a callback it behaves exactly like wxSplitterWindow.
class WxdSplitterWindow : public wxSplitterWindow {
public:
    WxdSplitterWindow(wxWindow* parent, wxWindowID id, const wxPoint& pos, const wxSize& size,
                      long style)
        : wxSplitterWindow(parent, id, pos, size, style)
    {
    }

    ~WxdSplitterWindow() override
    {
        ReleaseUserData();
    }

    void SetDrawSashCallback(wxd_SplitterWindow_DrawSash_Callback callback,
                             wxd_SplitterWindow_FreeUserData_Callback free_user_data, void* user_data)
    {
        ReleaseUserData();
        m_drawSash = callback;
        m_freeUserData = free_user_data;
        m_userData = user_data;
        Refresh();
    }

    void SetSashTrackerVisible(bool visible)
    {
        m_trackerVisible = visible;
    }

    void DrawSash(wxDC& dc) override
    {
        if (!m_drawSash) {
            wxSplitterWindow::DrawSash(dc);
            return;
        }
        if (!IsSplit())
            return;
        const wxSize client = GetClientSize();
        const int position = GetSashPosition();
        const int size = GetSashSize();
        const bool vertical = GetSplitMode() == wxSPLIT_VERTICAL;
        wxd_Rect sash = vertical ? wxd_Rect{ position, 0, size, client.y }
                                 : wxd_Rect{ 0, position, client.x, size };
        m_drawSash(m_userData, reinterpret_cast<wxd_DC_t*>(&dc), sash, vertical);
    }

    void DrawSashTracker(int x, int y) override
    {
        if (m_trackerVisible)
            wxSplitterWindow::DrawSashTracker(x, y);
    }

private:
    void ReleaseUserData()
    {
        if (m_freeUserData && m_userData)
            m_freeUserData(m_userData);
        m_freeUserData = nullptr;
        m_userData = nullptr;
        m_drawSash = nullptr;
    }

    wxd_SplitterWindow_DrawSash_Callback m_drawSash = nullptr;
    wxd_SplitterWindow_FreeUserData_Callback m_freeUserData = nullptr;
    void* m_userData = nullptr;
    bool m_trackerVisible = true;
};

// Implementation for wxd_SplitterWindow_Create
WXD_EXPORTED wxd_SplitterWindow_t*
wxd_SplitterWindow_Create(wxd_Window_t* parent, wxd_Id id, wxd_Point pos, wxd_Size size,
                          wxd_Style_t style)
{
    wxWindow* parentWin = reinterpret_cast<wxWindow*>(parent);
    // Need to ensure style includes wxSP_NOBORDER if parent is already providing border
    // or manage borders appropriately.
    wxSplitterWindow* splitter = new WxdSplitterWindow(parentWin, id, wxPoint(pos.x, pos.y),
                                                       wxSize(size.width, size.height), style);
    // Note: wxSplitterWindow manages its children, so attaching cleanup notifiers
    // to the *child panes* might be necessary if they aren't otherwise owned by Rust.
    // The splitter itself will be cleaned up by its parent.
    return reinterpret_cast<wxd_SplitterWindow_t*>(splitter);
}

// Implementation for wxd_SplitterWindow_SplitVertically
WXD_EXPORTED bool
wxd_SplitterWindow_SplitVertically(wxd_SplitterWindow_t* self, wxd_Window_t* window1,
                                   wxd_Window_t* window2, int sashPosition)
{
    wxSplitterWindow* splitter = reinterpret_cast<wxSplitterWindow*>(self);
    wxWindow* win1 = reinterpret_cast<wxWindow*>(window1);
    wxWindow* win2 = reinterpret_cast<wxWindow*>(window2);
    if (!splitter || !win1 || !win2)
        return false;
    return splitter->SplitVertically(win1, win2, sashPosition);
}

// Implementation for wxd_SplitterWindow_SplitHorizontally
WXD_EXPORTED bool
wxd_SplitterWindow_SplitHorizontally(wxd_SplitterWindow_t* self, wxd_Window_t* window1,
                                     wxd_Window_t* window2, int sashPosition)
{
    wxSplitterWindow* splitter = reinterpret_cast<wxSplitterWindow*>(self);
    wxWindow* win1 = reinterpret_cast<wxWindow*>(window1);
    wxWindow* win2 = reinterpret_cast<wxWindow*>(window2);
    if (!splitter || !win1 || !win2)
        return false;
    return splitter->SplitHorizontally(win1, win2, sashPosition);
}

// Implementation for wxd_SplitterWindow_Unsplit
WXD_EXPORTED bool
wxd_SplitterWindow_Unsplit(wxd_SplitterWindow_t* self, wxd_Window_t* toRemove)
{
    wxSplitterWindow* splitter = reinterpret_cast<wxSplitterWindow*>(self);
    wxWindow* winToRemove = reinterpret_cast<wxWindow*>(toRemove);
    if (!splitter)
        return false;
    // winToRemove can be null, in which case the inactive pane is removed.
    return splitter->Unsplit(winToRemove);
}

// Implementation for wxd_SplitterWindow_Initialize
WXD_EXPORTED void
wxd_SplitterWindow_Initialize(wxd_SplitterWindow_t* self, wxd_Window_t* window)
{
    wxSplitterWindow* splitter = reinterpret_cast<wxSplitterWindow*>(self);
    wxWindow* win = reinterpret_cast<wxWindow*>(window);
    if (!splitter || !win)
        return;
    splitter->Initialize(win);
}

// Implementation for wxd_SplitterWindow_SetSashPosition
WXD_EXPORTED void
wxd_SplitterWindow_SetSashPosition(wxd_SplitterWindow_t* self, int position, bool redraw)
{
    wxSplitterWindow* splitter = reinterpret_cast<wxSplitterWindow*>(self);
    if (!splitter)
        return;
    splitter->SetSashPosition(position, redraw);
}

// Implementation for wxd_SplitterWindow_GetSashPosition
WXD_EXPORTED int
wxd_SplitterWindow_GetSashPosition(wxd_SplitterWindow_t* self)
{
    wxSplitterWindow* splitter = reinterpret_cast<wxSplitterWindow*>(self);
    if (!splitter)
        return 0;
    return splitter->GetSashPosition();
}

// Implementation for wxd_SplitterWindow_SetMinimumPaneSize
WXD_EXPORTED void
wxd_SplitterWindow_SetMinimumPaneSize(wxd_SplitterWindow_t* self, int paneSize)
{
    wxSplitterWindow* splitter = reinterpret_cast<wxSplitterWindow*>(self);
    if (!splitter)
        return;
    splitter->SetMinimumPaneSize(paneSize);
}

// Implementation for wxd_SplitterEvent_GetSashPosition
WXD_EXPORTED int
wxd_SplitterEvent_GetSashPosition(wxd_Event_t* event)
{
    if (!event)
        return 0;
    wxSplitterEvent* splitterEvent =
        dynamic_cast<wxSplitterEvent*>(reinterpret_cast<wxEvent*>(event));
    if (!splitterEvent)
        return 0; // Or maybe -1 to indicate error?
    return splitterEvent->GetSashPosition();
}

WXD_EXPORTED bool
wxd_SplitterWindow_IsSplit(wxd_SplitterWindow_t* self)
{
    wxSplitterWindow* splitter = reinterpret_cast<wxSplitterWindow*>(self);
    return splitter ? splitter->IsSplit() : false;
}

WXD_EXPORTED bool
wxd_SplitterWindow_IsSplitVertically(wxd_SplitterWindow_t* self)
{
    wxSplitterWindow* splitter = reinterpret_cast<wxSplitterWindow*>(self);
    return splitter ? splitter->GetSplitMode() == wxSPLIT_VERTICAL : false;
}

WXD_EXPORTED int
wxd_SplitterWindow_GetSashSize(wxd_SplitterWindow_t* self)
{
    wxSplitterWindow* splitter = reinterpret_cast<wxSplitterWindow*>(self);
    return splitter ? splitter->GetSashSize() : 0;
}

WXD_EXPORTED void
wxd_SplitterWindow_SetSashGravity(wxd_SplitterWindow_t* self, double gravity)
{
    wxSplitterWindow* splitter = reinterpret_cast<wxSplitterWindow*>(self);
    if (splitter)
        splitter->SetSashGravity(gravity);
}

WXD_EXPORTED void
wxd_SplitterWindow_SetDrawSashCallback(wxd_SplitterWindow_t* self,
                                       wxd_SplitterWindow_DrawSash_Callback callback,
                                       wxd_SplitterWindow_FreeUserData_Callback free_user_data,
                                       void* user_data)
{
    WxdSplitterWindow* splitter =
        dynamic_cast<WxdSplitterWindow*>(reinterpret_cast<wxSplitterWindow*>(self));
    if (splitter) {
        splitter->SetDrawSashCallback(callback, free_user_data, user_data);
    }
    else if (free_user_data && user_data) {
        free_user_data(user_data);
    }
}

WXD_EXPORTED void
wxd_SplitterWindow_SetSashTrackerVisible(wxd_SplitterWindow_t* self, bool visible)
{
    WxdSplitterWindow* splitter =
        dynamic_cast<WxdSplitterWindow*>(reinterpret_cast<wxSplitterWindow*>(self));
    if (splitter)
        splitter->SetSashTrackerVisible(visible);
}
