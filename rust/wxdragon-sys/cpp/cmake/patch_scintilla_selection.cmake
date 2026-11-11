# The selection of wxStyledTextCtrl with rounded corners and without the leading (the extra
# space above and below the lines), drawn by Scintilla itself where it fills the translucent
# selection. Scintilla has no setting for it. Applied to the wxWidgets sources before they are
# configured, in place and once (the marker is the comment it adds); a version of Scintilla where
# the text to replace is not found stops the build with a message, never a silent half patch.
#
#   wxd_patch_scintilla_selection(<wxWidgets source dir>)
#
# What it changes in EditView.cxx:
#  - the pieces of the translucent selection are drawn by `WxdSelectionRectangle`: inside the
#    height of the text, with round corners; a column selection (rectangular) stays a plain
#    rectangle, so that its lines run into each other;
#  - when a line's break is selected too, the block of the text goes on over it (one block, not a
#    block and a separate small one for the break); a line with nothing selected but its break
#    (an empty one) gets its own block there.

function(wxd_patch_scintilla_selection wx_dir)
    set(file "${wx_dir}/src/stc/scintilla/src/EditView.cxx")
    if(NOT EXISTS "${file}")
        return()
    endif()
    file(READ "${file}" text)
    string(FIND "${text}" "wxDragon: rounded selection v2" done)
    if(NOT done EQUAL -1)
        return()
    endif()

    # A first version of this patch (v1) may be in the sources already: taken back first.
    string(FIND "${text}" "wxDragon: rounded selection." old)
    if(NOT old EQUAL -1)
        string(FIND "${text}" "// wxDragon: rounded selection." start)
        string(FIND "${text}" "static void DrawTranslucentSelection(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll," stop)
        math(EXPR length "${stop} - ${start}")
        string(SUBSTRING "${text}" ${start} ${length} v1)
        string(REPLACE "${v1}" "" text "${text}")
        string(REPLACE "WxdSelectionRectangle(surface, model, vsDraw, rcSelection, background, alpha);" "SimpleAlphaRectangle(surface, rcSelection, background, alpha);" text "${text}")
        string(REPLACE "WxdSelectionRectangle(surface, model, vsDraw, rcSegment, background, alpha);" "SimpleAlphaRectangle(surface, rcSegment, background, alpha);" text "${text}")
        string(REPLACE "WxdSelectionRectangle(surface, model, vsDraw, rcSegment, SelectionBackground(vsDraw, r == model.sel.Main(), model.primarySelection), alpha);" "SimpleAlphaRectangle(surface, rcSegment, SelectionBackground(vsDraw, r == model.sel.Main(), model.primarySelection), alpha);" text "${text}")
    endif()

    # 1. What Scintilla's code before the helpers needs to know: the two settings.
    set(anchor_eol [=[void EditView::DrawEOL(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll,]=])
    string(FIND "${text}" "${anchor_eol}" found)
    if(found EQUAL -1)
        message(FATAL_ERROR "wxDragon: this Scintilla has no DrawEOL as expected: the rounded selection patch (cmake/patch_scintilla_selection.cmake) needs to be looked at")
    endif()
    set(declarations [=[// wxDragon: rounded selection v2. Set from wxStyledTextCtrl::SetSelectionShape.
extern "C" {
extern int wxd_selection_radius;
extern int wxd_selection_inset;
}

static void WxdSelectionRectangle(Surface *surface, const EditModel &model, const ViewStyle &vsDraw,
	PRectangle rc, ColourDesired fill, int alpha);
static bool WxdSelectionShaped(const EditModel &model);
static void WxdEolSelection(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll,
	Sci::Position posLineStart, PRectangle rcSegment, ColourDesired fill, int alpha);

]=])
    string(REPLACE "${anchor_eol}" "${declarations}${anchor_eol}" text "${text}")

    # 2. The helpers, before the function that draws the translucent selection.
    set(anchor [=[static void DrawTranslucentSelection(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll,]=])
    string(FIND "${text}" "${anchor}" found)
    if(found EQUAL -1)
        message(FATAL_ERROR "wxDragon: this Scintilla has no DrawTranslucentSelection as expected: the rounded selection patch (cmake/patch_scintilla_selection.cmake) needs to be looked at")
    endif()
    set(helpers [=[int wxd_selection_radius = 0;
int wxd_selection_inset = 0;

// Shaped selection on: set, and not a column selection (which stays a plain rectangle).
static bool WxdSelectionShaped(const EditModel &model) {
	return (wxd_selection_radius > 0 || wxd_selection_inset) && !model.sel.IsRectangular();
}

static void WxdSelectionRectangle(Surface *surface, const EditModel &model, const ViewStyle &vsDraw,
	PRectangle rc, ColourDesired fill, int alpha) {
	if (alpha == SC_ALPHA_NOALPHA) {
		return;
	}
	if (!WxdSelectionShaped(model)) {
		surface->AlphaRectangle(rc, 0, fill, alpha, fill, alpha, 0);
		return;
	}
	if (wxd_selection_inset) {
		rc.top += vsDraw.extraAscent;
		rc.bottom -= vsDraw.extraDescent;
	}
	surface->AlphaRectangle(rc, wxd_selection_radius, fill, alpha, fill, 0, 0);
}

// The selected break of a line: with the shape on, it is part of the block of the text when
// there is selected text before it (see WxdSelectionText), and its own block when there is not.
static void WxdEolSelection(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll,
	Sci::Position posLineStart, PRectangle rcSegment, ColourDesired fill, int alpha) {
	if (!WxdSelectionShaped(model)) {
		SimpleAlphaRectangle(surface, rcSegment, fill, alpha);
		return;
	}
	const bool textSelected = (ll->numCharsBeforeEOL > 0) &&
		(model.sel.CharacterInSelection(posLineStart + ll->numCharsBeforeEOL - 1) != 0);
	if (!textSelected) {
		WxdSelectionRectangle(surface, model, vsDraw, rcSegment, fill, alpha);
	}
}

// The block of the selected text of a row: when the break of its line is selected too (and this is
// the last row of the line), it goes on over the break.
static void WxdSelectionText(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll,
	Sci::Line line, int subLine, Sci::Position posLineStart, Range lineRange, const SelectionSegment &portion,
	PRectangle rcSegment, PRectangle rcLine, ColourDesired fill, int alpha) {
	if (WxdSelectionShaped(model) && subLine == (ll->lines - 1) && line < model.pdoc->LinesTotal() - 1 &&
		portion.end.Position() == posLineStart + lineRange.end && !portion.end.VirtualSpace() &&
		model.sel.InSelectionForEOL(model.pdoc->LineStart(line + 1))) {
		rcSegment.right += vsDraw.aveCharWidth;
		if (rcSegment.right > rcLine.right) {
			rcSegment.right = rcLine.right;
		}
	}
	WxdSelectionRectangle(surface, model, vsDraw, rcSegment, fill, alpha);
}

]=])
    string(REPLACE "${anchor}" "${helpers}${anchor}" text "${text}")

    # 3. The places that fill a piece of the selection. Each pair is written out (the texts hold
    # semicolons, so they cannot travel in a list).
    set(from1 [=[SimpleAlphaRectangle(surface, rcSelection, background, alpha);]=])
    set(to1   [=[WxdSelectionRectangle(surface, model, vsDraw, rcSelection, background, alpha);]=])
    set(from2 [=[SimpleAlphaRectangle(surface, rcSegment, background, alpha);]=])
    set(to2   [=[WxdSelectionRectangle(surface, model, vsDraw, rcSegment, background, alpha);]=])
    set(from3 [=[SimpleAlphaRectangle(surface, rcSegment, SelectionBackground(vsDraw, r == model.sel.Main(), model.primarySelection), alpha);]=])
    set(to3   [=[WxdSelectionText(surface, model, vsDraw, ll, line, subLine, posLineStart, lineRange, portion, rcSegment, rcLine, SelectionBackground(vsDraw, r == model.sel.Main(), model.primarySelection), alpha);]=])
    set(from4 [=[SimpleAlphaRectangle(surface, rcSegment, SelectionBackground(vsDraw, eolInSelection == 1, model.primarySelection), alpha);]=])
    set(to4   [=[WxdEolSelection(surface, model, vsDraw, ll, model.pdoc->LineStart(line), rcSegment, SelectionBackground(vsDraw, eolInSelection == 1, model.primarySelection), alpha);]=])
    foreach(n 1 2 3 4)
        string(FIND "${text}" "${from${n}}" found)
        if(found EQUAL -1)
            message(FATAL_ERROR "wxDragon: the rounded selection patch did not find: ${from${n}}")
        endif()
        string(REPLACE "${from${n}}" "${to${n}}" text "${text}")
    endforeach()

    file(WRITE "${file}" "${text}")
    message(STATUS "wxDragon: rounded selection patch applied to ${file}")
endfunction()
