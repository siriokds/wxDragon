#ifndef WXD_EVENTFILTER_H
#define WXD_EVENTFILTER_H

#include "../wxd_types.h"

#ifdef __cplusplus
extern "C" {
#endif

// Called for every event the application processes, before any handler.
// Returns WXD_EVENT_FILTER_SKIP (-1) to let the event go on as usual,
// WXD_EVENT_FILTER_IGNORE (0) or WXD_EVENT_FILTER_PROCESSED (1) as in
// wxEventFilter::FilterEvent.
typedef int (*wxd_EventFilterCallback)(void* user_data, wxd_Event_t* event);

enum {
    WXD_EVENT_FILTER_SKIP = -1,
    WXD_EVENT_FILTER_IGNORE = 0,
    WXD_EVENT_FILTER_PROCESSED = 1
};

// Installs a global wxEventFilter (wxEvtHandler::AddFilter) calling
// `callback(user_data, event)`. `user_data` stays owned by the caller and
// must outlive the filter.
WXD_EXPORTED wxd_EventFilter_t*
wxd_EventFilter_Add(wxd_EventFilterCallback callback, void* user_data);

// Removes and deletes a filter made by wxd_EventFilter_Add.
WXD_EXPORTED void
wxd_EventFilter_Remove(wxd_EventFilter_t* filter);

#ifdef __cplusplus
}
#endif

#endif // WXD_EVENTFILTER_H
