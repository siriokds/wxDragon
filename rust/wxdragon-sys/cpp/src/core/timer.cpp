#include <wx/wxprec.h>
#include <wx/wx.h>
#include "../../include/wxdragon.h"
#include "../../include/core/wxd_timer.h"
#include <wx/timer.h>

namespace {

// A wxTimer that stops for good when its owner window is destroyed.
//
// In C++ a timer is usually a member of its window and dies with it. From
// Rust the timer's lifetime is separate: it can sit in shared state, in a
// closure, or be leaked on purpose, and keep running after the window is
// gone. Its next tick then delivers the event to the freed owner
// (wxTimerImpl::SendEvent -> SafelyProcessEvent on a dangling pointer) and
// the process crashes, typically while the application is quitting.
class wxdTimer : public wxTimer
{
public:
    explicit wxdTimer(wxEvtHandler* owner)
        : wxTimer(owner)
    {
        m_window = wxDynamicCast(owner, wxWindow);
        if (m_window)
            m_window->Bind(wxEVT_DESTROY, &wxdTimer::OnOwnerDestroy, this);
    }

    ~wxdTimer() override
    {
        if (m_window && !m_ownerGone)
            m_window->Unbind(wxEVT_DESTROY, &wxdTimer::OnOwnerDestroy, this);
    }

    bool Start(int milliseconds = -1, bool oneShot = wxTIMER_CONTINUOUS) override
    {
        // Nobody would receive the event any more.
        if (m_ownerGone)
            return false;
        return wxTimer::Start(milliseconds, oneShot);
    }

private:
    void OnOwnerDestroy(wxWindowDestroyEvent& event)
    {
        // Children's destroy events can reach the owner too: only the owner's
        // own destruction counts.
        if (event.GetEventObject() == m_window) {
            Stop();
            m_ownerGone = true;
        }
        event.Skip();
    }

    wxWindow* m_window = nullptr;
    bool m_ownerGone = false;
};

} // namespace

extern "C" {

// Create a new wxTimer associated with an event handler. If the owner is a
// window, the timer stops when the window is destroyed (see wxdTimer).
WXD_EXPORTED wxd_Timer_t*
wxd_Timer_Create(wxd_EvtHandler_t* owner)
{
    wxEvtHandler* wx_owner = reinterpret_cast<wxEvtHandler*>(owner);
    if (!wx_owner) {
        return nullptr;
    }
    wxTimer* timer = new wxdTimer(wx_owner);
    return reinterpret_cast<wxd_Timer_t*>(timer);
}

// Destroy/delete a wxTimer
WXD_EXPORTED void
wxd_Timer_Destroy(wxd_Timer_t* self)
{
    if (!self)
        return;
    wxTimer* timer = reinterpret_cast<wxTimer*>(self);
    if (timer->IsRunning()) {
        timer->Stop();
    }
    // ~wxEvtHandler deletes the client object holding the bound Rust closures.
    delete timer;
}

// Get the timer ID used when delivering events to its owner
WXD_EXPORTED int
wxd_Timer_GetId(wxd_Timer_t* self)
{
    if (!self)
        return wxID_ANY;
    wxTimer* timer = reinterpret_cast<wxTimer*>(self);
    return timer->GetId();
}

// Start the timer
WXD_EXPORTED bool
wxd_Timer_Start(wxd_Timer_t* self, int milliseconds, bool oneShot)
{
    if (!self)
        return false;
    wxTimer* timer = reinterpret_cast<wxTimer*>(self);
    return timer->Start(milliseconds, oneShot);
}

// Stop the timer
WXD_EXPORTED void
wxd_Timer_Stop(wxd_Timer_t* self)
{
    if (!self)
        return;
    wxTimer* timer = reinterpret_cast<wxTimer*>(self);
    timer->Stop();
}

// Check if the timer is running
WXD_EXPORTED bool
wxd_Timer_IsRunning(wxd_Timer_t* self)
{
    if (!self)
        return false;
    wxTimer* timer = reinterpret_cast<wxTimer*>(self);
    return timer->IsRunning();
}

// Get the timer interval in milliseconds
WXD_EXPORTED int
wxd_Timer_GetInterval(wxd_Timer_t* self)
{
    if (!self)
        return 0;
    wxTimer* timer = reinterpret_cast<wxTimer*>(self);
    return timer->GetInterval();
}

// Set the timer interval in milliseconds (restart with new interval if running)
WXD_EXPORTED void
wxd_Timer_SetInterval(wxd_Timer_t* self, int milliseconds)
{
    if (!self)
        return;
    wxTimer* timer = reinterpret_cast<wxTimer*>(self);
    bool wasRunning = timer->IsRunning();
    bool oneShot = timer->IsOneShot();

    if (wasRunning) {
        timer->Stop();
        timer->Start(milliseconds, oneShot);
    }
    // If not running, the interval will be used on the next Start() call
}

} // extern "C"
