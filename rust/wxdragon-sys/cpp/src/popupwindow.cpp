#include <wx/wxprec.h>
#include <wx/wx.h>
#include <wx/popupwin.h>
#include "wxdragon.h"

extern "C" {

WXD_EXPORTED wxd_PopupWindow_t*
wxd_PopupWindow_Create(wxd_Window_t* parent, wxd_Style_t style)
{
    wxWindow* parentWin = reinterpret_cast<wxWindow*>(parent);
    return reinterpret_cast<wxd_PopupWindow_t*>(new wxPopupWindow(parentWin, style));
}

WXD_EXPORTED wxd_PopupWindow_t*
wxd_PopupWindow_CreateTransparent(wxd_Window_t* parent, wxd_Style_t style)
{
    wxWindow* parentWin = reinterpret_cast<wxWindow*>(parent);
    // Asked to the parent: on GTK the question needs a created window, and
    // the style must be set before creation.
    if (!parentWin || !parentWin->IsTransparentBackgroundSupported())
        return nullptr;
    auto* popup = new wxPopupWindow();
    if (!popup->SetBackgroundStyle(wxBG_STYLE_TRANSPARENT) ||
        !popup->Create(parentWin, style))
    {
        delete popup;
        return nullptr;
    }
    return reinterpret_cast<wxd_PopupWindow_t*>(popup);
}

} // extern "C"
