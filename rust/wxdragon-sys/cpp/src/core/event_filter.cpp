#include <wx/wxprec.h>
#include <wx/wx.h>
#include <wx/eventfilter.h>
#include "wxdragon.h"

namespace {

class wxdEventFilter : public wxEventFilter
{
public:
    wxdEventFilter(wxd_EventFilterCallback callback, void* user_data)
        : m_callback(callback), m_userData(user_data)
    {
    }

    int FilterEvent(wxEvent& event) override
    {
        return m_callback(m_userData, reinterpret_cast<wxd_Event_t*>(&event));
    }

private:
    wxd_EventFilterCallback m_callback;
    void* m_userData;
};

} // namespace

extern "C" {

WXD_EXPORTED wxd_EventFilter_t*
wxd_EventFilter_Add(wxd_EventFilterCallback callback, void* user_data)
{
    if (!callback)
        return nullptr;
    auto* filter = new wxdEventFilter(callback, user_data);
    wxEvtHandler::AddFilter(filter);
    return reinterpret_cast<wxd_EventFilter_t*>(filter);
}

WXD_EXPORTED void
wxd_EventFilter_Remove(wxd_EventFilter_t* filter)
{
    if (!filter)
        return;
    auto* f = reinterpret_cast<wxdEventFilter*>(filter);
    wxEvtHandler::RemoveFilter(f);
    delete f;
}

} // extern "C"
