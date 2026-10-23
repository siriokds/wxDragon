#ifndef WXD_DISPATCH_H
#define WXD_DISPATCH_H

#include <wx/window.h>
#include <wx/weakref.h>

#include <vector>

// A window is never deleted while one of its events, or an event of a window
// inside it, is still being dispatched to Rust.
//
// Destruction is already deferred (wxd_Window_Destroy defers child windows,
// top-level windows wait in wxPendingDelete for idle), but a callback that
// opens a modal dialog runs a nested event loop, and the deferred destruction
// happens there, under the callback: wx keeps processing the event on the dead
// window when the callback returns, and the callback is freed while it runs.
// So the deferred destruction also waits for the dispatches inside the window
// to unwind.
//
// Main thread only, like every wx window call.
namespace wxd_dispatch {

// Around every call into Rust (WxdDispatchGuard in event.cpp).
void
Enter(wxEvtHandler* owner);
void
Leave();

// `owner` is being destroyed: it no longer counts as dispatching.
void
Forget(wxEvtHandler* owner);

// `window`, or a window inside it, has a dispatch on the stack.
bool
IsBusy(wxWindow* window);

// Hides a child window now and deletes it at idle, once nothing inside it is
// being dispatched.
void
ScheduleDestroy(wxWindow* window);

// At idle: deletes the scheduled child windows that are no longer busy.
void
DestroyWhatIsSafe();

// Around wxApp::ProcessIdle: keeps the busy top-level windows out of
// wxPendingDelete while it is processed, and puts them back afterwards.
class PendingDeleteHold {
public:
    PendingDeleteHold();
    ~PendingDeleteHold();

    PendingDeleteHold(const PendingDeleteHold&) = delete;
    PendingDeleteHold&
    operator=(const PendingDeleteHold&) = delete;

private:
    std::vector<wxWeakRef<wxWindow>> m_held;
};

} // namespace wxd_dispatch

#endif // WXD_DISPATCH_H
