// Parts of wxStyledTextCtrl that Scintilla's own messages do not reach: they use Scintilla's
// internals (the view of the editor), taken from the wxWidgets source the library is built from.
//
// - The mark of a tab (View > Show Whitespace). Scintilla draws a long arrow as tall as the line,
//   up to the next letter, with no setting for the head, the gap or the line. Its editor view has a
//   hook for it (`customDrawTabArrow`, made for platforms that cannot draw the arrow themselves);
//   this sets it: a line with a small head, kept off the edges of the tab.

#include "wxd_dip.h"
#include "wx/wxprec.h"

#ifndef WX_PRECOMP
#include "wx/wx.h"
#endif

#include "../include/wxdragon.h"
#if wxUSE_STC
#include "wx/stc/stc.h"
#include "ScintillaWX.h"

using namespace Scintilla;

namespace
{
struct TabArrowStyle
{
    int head = 3;
    int gap_start = 3;
    int gap_end = 5;
};

TabArrowStyle g_tab_arrow;

// The view of the editor is protected in `Editor`; a class derived from `ScintillaWX` may name it.
struct Peek : public ScintillaWX
{
    static EditView& ViewOf(ScintillaWX* s) { return static_cast<Peek*>(s)->view; }
};

// The pen colour is already the whitespace colour; the rectangle is the tab's own cell, `ymid` the
// middle of the line.
void DrawTab(Surface* surface, PRectangle rcTab, int ymid)
{
    int x1 = static_cast<int>(rcTab.left) + g_tab_arrow.gap_start;
    int x2 = static_cast<int>(rcTab.right) - g_tab_arrow.gap_end;
    if (x2 <= x1) {
        // A cell too narrow for the gaps: what there is.
        x1 = static_cast<int>(rcTab.left);
        x2 = static_cast<int>(rcTab.right);
    }
    surface->MoveTo(x1, ymid);
    surface->LineTo(x2, ymid);
    int head = g_tab_arrow.head;
    if (head > 0 && x2 - x1 >= head) {
        surface->LineTo(x2 - head, ymid - head);
        surface->MoveTo(x2, ymid);
        surface->LineTo(x2 - head, ymid + head);
    }
}
} // namespace

extern "C" {

// The tab mark of every wxStyledTextCtrl drawn from now on: a line with a head `head` pixels
// high, starting `gap_start` pixels after the tab's left edge and ending `gap_end` before its
// right edge (the letter that follows). `head` 0: the line alone. The setting is shared by all
// editors of the program, as the hook of Scintilla takes no data.
WXD_EXPORTED void
wxd_StyledTextCtrl_SetTabArrow(wxd_StyledTextCtrl_t* self, int head, int gap_start, int gap_end)
{
    wxStyledTextCtrl* ctrl = (wxStyledTextCtrl*)self;
    if (!ctrl) {
        return;
    }
    g_tab_arrow.head = head < 0 ? 0 : head;
    g_tab_arrow.gap_start = gap_start < 0 ? 0 : gap_start;
    g_tab_arrow.gap_end = gap_end < 0 ? 0 : gap_end;
    ScintillaWX* editor = static_cast<ScintillaWX*>(ctrl->GetDirectPointer());
    if (editor) {
        Peek::ViewOf(editor).customDrawTabArrow = DrawTab;
        ctrl->Refresh();
    }
}

} // extern "C"
#endif // wxUSE_STC
