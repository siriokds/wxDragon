#include <wx/wxprec.h>
#include <wx/wx.h>
#include <wx/app.h>
#include <wx/weakref.h>

#include "wxd_dispatch.h"

#include <vector>

namespace {

// The windows whose events are being dispatched to Rust, innermost last; null
// for an owner that is not a window, or one that has been destroyed since.
std::vector<wxWindow*> g_active;

// Child windows waiting to be deleted.
std::vector<wxWeakRef<wxWindow>> g_doomed;

// Something waits for a dispatch to unwind: wake the idle loop when one does.
bool g_waiting = false;

} // namespace

namespace wxd_dispatch {

void
Enter(wxEvtHandler* owner)
{
    g_active.push_back(wxDynamicCast(owner, wxWindow));
}

void
Leave()
{
    if (!g_active.empty()) {
        g_active.pop_back();
    }
    if (g_waiting) {
        g_waiting = false;
        wxWakeUpIdle();
    }
}

void
Forget(wxEvtHandler* owner)
{
    for (wxWindow*& window : g_active) {
        if (window != nullptr && static_cast<wxEvtHandler*>(window) == owner) {
            window = nullptr;
        }
    }
}

bool
IsBusy(wxWindow* window)
{
    for (wxWindow* active : g_active) {
        // All the way up: deleting a window deletes everything it owns,
        // dialogs (top-level children) included.
        for (wxWindow* w = active; w != nullptr; w = w->GetParent()) {
            if (w == window) {
                return true;
            }
        }
    }
    return false;
}

void
ScheduleDestroy(wxWindow* window)
{
    for (const wxWeakRef<wxWindow>& doomed : g_doomed) {
        if (doomed.get() == window) {
            return;
        }
    }
    // Gone from the screen at once, as Destroy() does with top-level windows.
    window->Hide();
    g_doomed.emplace_back(window);
    wxWakeUpIdle();
}

void
DestroyWhatIsSafe()
{
    // Deleting a window sends events, and their callbacks may schedule more.
    std::vector<wxWeakRef<wxWindow>> batch;
    batch.swap(g_doomed);
    for (wxWeakRef<wxWindow>& doomed : batch) {
        wxWindow* window = doomed.get();
        if (window == nullptr) {
            continue; // Already deleted with its parent.
        }
        if (IsBusy(window)) {
            g_doomed.push_back(doomed);
            g_waiting = true;
            continue;
        }
        window->Destroy();
    }
}

PendingDeleteHold::PendingDeleteHold()
{
    // A busy top-level window is held back with all its busy ancestors (they
    // are busy too), so none of the held windows is deleted while held.
    wxList::compatibility_iterator node = wxPendingDelete.GetFirst();
    while (node) {
        wxList::compatibility_iterator next = node->GetNext();
        wxWindow* window = wxDynamicCast(node->GetData(), wxWindow);
        if (window != nullptr && IsBusy(window)) {
            m_held.emplace_back(window);
            wxPendingDelete.Erase(node);
        }
        node = next;
    }
    if (!m_held.empty()) {
        g_waiting = true;
    }
}

PendingDeleteHold::~PendingDeleteHold()
{
    for (wxWeakRef<wxWindow>& held : m_held) {
        wxWindow* window = held.get();
        if (window != nullptr && !wxPendingDelete.Member(window)) {
            wxPendingDelete.Append(window);
        }
    }
}

} // namespace wxd_dispatch
