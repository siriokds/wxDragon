#include <wx/wxprec.h>
#include <wx/wx.h>
#include "wx/panel.h"
#include "wx/window.h" // Include parent class header if needed
#include "wxdragon.h"

namespace {

// A wxPanel that can be told not to take the keyboard focus. A drawn button
// (a panel with a click handler) must not keep the focus after a click: on
// macOS the click makes it the first responder, and typing then goes nowhere
// (the system beeps) until the user clicks back into the real control.
// wxWindow::AcceptsFocus() is the hook; a panel has no public way to turn its
// own focus off.
class wxdPanel : public wxPanel
{
public:
    using wxPanel::wxPanel;

    bool AcceptsFocus() const override
    {
        return m_acceptsFocus && wxPanel::AcceptsFocus();
    }

    bool AcceptsFocusFromKeyboard() const override
    {
        return m_acceptsFocus && wxPanel::AcceptsFocusFromKeyboard();
    }

    bool m_acceptsFocus = true;
};

} // namespace

extern "C" {

// Refuse (or allow again) the keyboard focus. Panels not made by
// wxd_Panel_Create (e.g. from XRC) are left unchanged.
WXD_EXPORTED void
wxd_Panel_SetAcceptsFocus(wxd_Panel_t* self, bool accepts)
{
    if (auto* panel = dynamic_cast<wxdPanel*>(reinterpret_cast<wxPanel*>(self)))
        panel->m_acceptsFocus = accepts;
}

// Implement the creation function for wxPanel
WXD_EXPORTED wxd_Panel_t*
wxd_Panel_Create(wxd_Window_t* parent, wxd_Id id, wxd_Point pos, wxd_Size size, wxd_Style_t style)
{
    wxWindow* parentWin = (wxWindow*)parent; // Cast opaque parent pointer

    // Create wxWidgets objects for position and size
    wxPoint wxPos(pos.x, pos.y);
    wxSize wxSize(size.width, size.height);

    // Create the wxPanel instance
    // Note: wxPanel inherits from wxWindow, so the wxd_Window_t* parent is appropriate.
    wxPanel* panel = new wxdPanel(parentWin, id, wxPos, wxSize, style);

    // Return the created panel pointer, cast to the opaque C type
    return (wxd_Panel_t*)panel;
}

// TODO: Add wxd_Panel_Destroy if needed, or confirm wxWindow_Destroy works.
// Panels, like most widgets, are typically destroyed when their parent frame/dialog is destroyed.
// Explicit destruction might only be necessary if dynamically removing panels without destroying the parent.
// For now, we assume parent-based destruction.

} // extern "C"