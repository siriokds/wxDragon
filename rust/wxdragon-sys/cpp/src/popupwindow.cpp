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

} // extern "C"
