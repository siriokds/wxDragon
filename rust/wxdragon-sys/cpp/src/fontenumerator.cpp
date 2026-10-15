#include <wx/wxprec.h>
#include <wx/wx.h>
#include <wx/fontenum.h>
#include "../include/wxdragon.h"
#include "wxd_utils.h"

extern "C" {

WXD_EXPORTED wxd_ArrayString_t*
wxd_FontEnumerator_GetFacenames(bool fixed_width_only)
{
    wxArrayString* names = new (std::nothrow) wxArrayString();
    if (!names)
        return nullptr;
    *names = wxFontEnumerator::GetFacenames(wxFONTENCODING_SYSTEM, fixed_width_only);
    return reinterpret_cast<wxd_ArrayString_t*>(names);
}

WXD_EXPORTED bool
wxd_FontEnumerator_IsValidFacename(const char* facename)
{
    if (!facename)
        return false;
    return wxFontEnumerator::IsValidFacename(wxString::FromUTF8(facename));
}

} // extern "C"
