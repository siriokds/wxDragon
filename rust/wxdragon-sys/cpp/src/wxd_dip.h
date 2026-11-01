// wxDragon speaks device-independent pixels (DIPs) in both directions.
//
// Every coordinate that enters wx from Rust goes through FromDIP and every
// coordinate wx hands back goes through ToDIP, using the window the value
// belongs to. On macOS and GTK wx already works in logical units, so these
// are identities there; on Windows (per-monitor DPI aware) they turn the
// points the application writes into physical pixels and back. Without this
// only the values passed at creation were converted (wxd_utils.h to_wx), and
// a size read from one call and written to another grew by the DPI scale.
//
// wxDefaultCoord (-1) and wxDefaultSize/wxDefaultPosition pass unchanged:
// they mean "let wx decide", not a length.
#pragma once

#include <wx/window.h>
#include <wx/dc.h>
#include "../include/wxd_types.h"

namespace wxd_dip {

// One length into wx (DIPs → pixels for `w`).
inline int
in(const wxWindow* w, int v)
{
    return (w && v != -1) ? w->FromDIP(v) : v;
}

// One length out of wx (pixels → DIPs for `w`).
inline int
out(const wxWindow* w, int v)
{
    return (w && v != -1) ? w->ToDIP(v) : v;
}

// A coordinate that may be negative (screen positions left of the primary
// display): only the pair (-1, -1) is wxDefaultPosition.
inline wxPoint
in(const wxWindow* w, const wxd_Point& p)
{
    if (!w || (p.x == -1 && p.y == -1))
        return wxPoint(p.x, p.y);
    return w->FromDIP(wxPoint(p.x, p.y));
}

inline wxd_Point
out(const wxWindow* w, const wxPoint& p)
{
    if (!w || p == wxDefaultPosition)
        return { p.x, p.y };
    wxPoint r = w->ToDIP(p);
    return { r.x, r.y };
}

inline wxSize
in(const wxWindow* w, const wxd_Size& s)
{
    return wxSize(in(w, s.width), in(w, s.height));
}

inline wxd_Size
out(const wxWindow* w, const wxSize& s)
{
    return { out(w, s.GetWidth()), out(w, s.GetHeight()) };
}

inline wxRect
in(const wxWindow* w, const wxd_Rect& r)
{
    if (!w)
        return wxRect(r.x, r.y, r.width, r.height);
    wxPoint p = w->FromDIP(wxPoint(r.x, r.y));
    return wxRect(p.x, p.y, in(w, r.width), in(w, r.height));
}

inline wxd_Rect
out(const wxWindow* w, const wxRect& r)
{
    if (!w)
        return { r.x, r.y, r.width, r.height };
    wxPoint p = w->ToDIP(r.GetPosition());
    return { p.x, p.y, out(w, r.width), out(w, r.height) };
}

// The same for a device context (wxDC::FromDIP/ToDIP use the DC's window).
inline int
in(const wxDC* dc, int v)
{
    return dc ? dc->FromDIP(v) : v;
}

inline int
out(const wxDC* dc, int v)
{
    return dc ? dc->ToDIP(v) : v;
}

} // namespace wxd_dip

#include <wx/display.h>

namespace wxd_dip {

// Screen coordinates (displays, the global mouse position). On Windows every
// display has its own scale and the virtual screen is in physical pixels; a
// screen point in DIPs is the pixel point divided by the scale of the display
// it falls on. On macOS and GTK screen coordinates are already logical, and
// wxDisplay::GetScaleFactor is the backing scale (2 on Retina): nothing to do.
inline wxPoint
screen_in(const wxd_Point& p)
{
#ifdef __WXMSW__
    for (unsigned int i = 0; i < wxDisplay::GetCount(); ++i) {
        wxDisplay d(i);
        double s = d.GetScaleFactor();
        wxPoint px(wxRound(p.x * s), wxRound(p.y * s));
        if (d.GetGeometry().Contains(px))
            return px;
    }
    double s = wxDisplay().GetScaleFactor();
    return wxPoint(wxRound(p.x * s), wxRound(p.y * s));
#else
    return wxPoint(p.x, p.y);
#endif
}

inline double
screen_scale_at(const wxPoint& px)
{
#ifdef __WXMSW__
    int i = wxDisplay::GetFromPoint(px);
    return (i == wxNOT_FOUND ? wxDisplay() : wxDisplay(i)).GetScaleFactor();
#else
    (void)px;
    return 1.0;
#endif
}

inline wxd_Point
screen_out(const wxPoint& px)
{
    double s = screen_scale_at(px);
    return { wxRound(px.x / s), wxRound(px.y / s) };
}

// A display's own rectangles, divided by its own scale.
inline wxd_Rect
display_out(const wxDisplay& d, const wxRect& r)
{
#ifdef __WXMSW__
    double s = d.GetScaleFactor();
    return { wxRound(r.x / s), wxRound(r.y / s), wxRound(r.width / s), wxRound(r.height / s) };
#else
    (void)d;
    return { r.x, r.y, r.width, r.height };
#endif
}

} // namespace wxd_dip

namespace wxd_dip {

// A length with no window to ask (sizer borders and gaps: a sizer may not
// belong to a window yet). The primary display's scale, like the values
// passed at creation (wxd_utils.h to_wx). Unchanged on macOS and GTK.
inline int
any(int v)
{
    return v == -1 ? v : wxWindow::FromDIP(wxSize(v, v), nullptr).GetWidth();
}

} // namespace wxd_dip

#include <vector>

namespace wxd_dip {

// A rectangle into a DC, converted by its edges: two rows of 20 DIPs at 175%
// end and start on the same pixel (35 + 35) instead of leaving a one-pixel
// gap or overlap from rounding each width alone.
inline wxRect
rect_in(const wxDC* dc, int x, int y, int w, int h)
{
    int x0 = in(dc, x), y0 = in(dc, y);
    return wxRect(x0, y0, in(dc, x + w) - x0, in(dc, y + h) - y0);
}

// A pen width: a hairline stays one pixel (sharp, as native controls draw
// it); thicker pens scale.
inline int
pen_in(const wxDC* dc, int width)
{
    return width <= 1 ? width : in(dc, width);
}

// The DC's scale for values that are not whole lengths (radii).
inline double
scale(const wxDC* dc)
{
    return dc ? dc->FromDIP(1000) / 1000.0 : 1.0;
}

inline std::vector<wxPoint>
points_in(const wxDC* dc, int n, const wxd_Point* pts)
{
    std::vector<wxPoint> v;
    v.reserve(n > 0 ? n : 0);
    for (int i = 0; i < n; ++i)
        v.emplace_back(in(dc, pts[i].x), in(dc, pts[i].y));
    return v;
}

} // namespace wxd_dip
