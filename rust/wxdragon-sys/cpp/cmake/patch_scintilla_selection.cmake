# The selection of wxStyledTextCtrl as a line is drawn in a text editor of today: round corners,
# as high as the text (without the leading, the extra space above and below the lines), and a
# line that is selected to its break selected across the whole column, as WhyNote's. Scintilla
# has no setting for it: it is drawn by Scintilla itself, where it fills the translucent
# selection, by a change to its sources.
#
# Applied to the wxWidgets sources before they are configured, in place and once. The first time
# the original `EditView.cxx` is kept beside it (`EditView.cxx.orig`) and every later version of
# the patch starts again from that one. A version of Scintilla where the text to replace is not
# found stops the build with a message, never a silent half patch.
#
#   wxd_patch_scintilla_selection(<wxWidgets source dir>)
#
# What it changes in EditView.cxx:
#  - the pieces of the translucent selection are drawn by `WxdSelectionRectangle`: inside the
#    height of the text, with round corners; a column selection (rectangular) stays a plain
#    rectangle, so that its lines run into each other;
#  - when the break of a line is selected too, the block goes on over it to the right edge of the
#    column (one block for the line); a line with nothing selected but its break (an empty one)
#    gets the block from where its text would start.

# The sources as they were before any version of the patch (v1, v2: no `.orig` was kept).
function(wxd_scintilla_pristine text out)
    set(t "${text}")
    string(FIND "${t}" "// wxDragon: rounded selection" at)
    if(at EQUAL -1)
        set(${out} "${t}" PARENT_SCOPE)
        return()
    endif()
    # The block of declarations (before DrawEOL) and of helpers (before DrawTranslucentSelection).
    string(FIND "${t}" "// wxDragon: rounded selection v2." start)
    if(NOT start EQUAL -1)
        string(FIND "${t}" "void EditView::DrawEOL(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll," stop)
        math(EXPR length "${stop} - ${start}")
        string(SUBSTRING "${t}" ${start} ${length} block)
        string(REPLACE "${block}" "" t "${t}")
        string(FIND "${t}" "int wxd_selection_radius = 0;" start)
        string(FIND "${t}" "static void DrawTranslucentSelection(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll," stop)
        math(EXPR length "${stop} - ${start}")
        string(SUBSTRING "${t}" ${start} ${length} block)
        string(REPLACE "${block}" "" t "${t}")
        string(REPLACE "WxdEolSelection(surface, model, vsDraw, ll, model.pdoc->LineStart(line), rcSegment, SelectionBackground(vsDraw, eolInSelection == 1, model.primarySelection), alpha);" "SimpleAlphaRectangle(surface, rcSegment, SelectionBackground(vsDraw, eolInSelection == 1, model.primarySelection), alpha);" t "${t}")
        string(REPLACE "WxdSelectionText(surface, model, vsDraw, ll, line, subLine, posLineStart, lineRange, portion, rcSegment, rcLine, SelectionBackground(vsDraw, r == model.sel.Main(), model.primarySelection), alpha);" "SimpleAlphaRectangle(surface, rcSegment, SelectionBackground(vsDraw, r == model.sel.Main(), model.primarySelection), alpha);" t "${t}")
    else()
        # v1: one helper before DrawTranslucentSelection, three calls.
        string(FIND "${t}" "// wxDragon: rounded selection." start)
        string(FIND "${t}" "static void DrawTranslucentSelection(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll," stop)
        math(EXPR length "${stop} - ${start}")
        string(SUBSTRING "${t}" ${start} ${length} block)
        string(REPLACE "${block}" "" t "${t}")
        string(REPLACE "WxdSelectionRectangle(surface, model, vsDraw, rcSegment, SelectionBackground(vsDraw, r == model.sel.Main(), model.primarySelection), alpha);" "SimpleAlphaRectangle(surface, rcSegment, SelectionBackground(vsDraw, r == model.sel.Main(), model.primarySelection), alpha);" t "${t}")
    endif()
    string(REPLACE "WxdSelectionRectangle(surface, model, vsDraw, rcSelection, background, alpha);" "SimpleAlphaRectangle(surface, rcSelection, background, alpha);" t "${t}")
    string(REPLACE "WxdSelectionRectangle(surface, model, vsDraw, rcSegment, background, alpha);" "SimpleAlphaRectangle(surface, rcSegment, background, alpha);" t "${t}")
    set(${out} "${t}" PARENT_SCOPE)
endfunction()

function(wxd_patch_scintilla_selection wx_dir)
    set(file "${wx_dir}/src/stc/scintilla/src/EditView.cxx")
    set(orig "${file}.orig")
    if(NOT EXISTS "${file}")
        return()
    endif()
    file(READ "${file}" current)
    string(FIND "${current}" "wxDragon: rounded selection v3" done)
    if(NOT done EQUAL -1)
        return()
    endif()
    if(EXISTS "${orig}")
        file(READ "${orig}" text)
    else()
        wxd_scintilla_pristine("${current}" text)
        file(WRITE "${orig}" "${text}")
    endif()

    # 1. What the code before the helpers needs to know: the two settings, and the helpers.
    set(anchor_eol [=[void EditView::DrawEOL(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll,]=])
    string(FIND "${text}" "${anchor_eol}" found)
    if(found EQUAL -1)
        message(FATAL_ERROR "wxDragon: this Scintilla has no DrawEOL as expected: the rounded selection patch (cmake/patch_scintilla_selection.cmake) needs to be looked at")
    endif()
    set(declarations [=[// wxDragon: rounded selection v3. Set from wxStyledTextCtrl::SetSelectionShape.
extern "C" {
extern int wxd_selection_radius;
extern int wxd_selection_inset;
}

static void WxdSelectionRectangle(Surface *surface, const EditModel &model, const ViewStyle &vsDraw,
	PRectangle rc, ColourDesired fill, int alpha);
static bool WxdSelectionShaped(const EditModel &model);
static void WxdEolSelection(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll,
	Sci::Position posLineStart, PRectangle rcLine, PRectangle rcSegment, ColourDesired fill, int alpha);

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

// The right edge of the column of text: where the block of a line selected to its break ends.
static XYPOSITION WxdColumnRight(const ViewStyle &vsDraw, PRectangle rcLine) {
	const XYPOSITION right = rcLine.right - vsDraw.rightMarginWidth;
	return (right > rcLine.left) ? right : rcLine.right;
}

// The selected break of a line. With the shape on, when there is selected text before it, the
// block of the text already goes over it (see WxdSelectionText); when there is not (an empty
// line, or the selection starts at the break), the block is drawn here, to the right edge.
static void WxdEolSelection(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll,
	Sci::Position posLineStart, PRectangle rcLine, PRectangle rcSegment, ColourDesired fill, int alpha) {
	if (!WxdSelectionShaped(model)) {
		SimpleAlphaRectangle(surface, rcSegment, fill, alpha);
		return;
	}
	const bool textSelected = (ll->numCharsBeforeEOL > 0) &&
		(model.sel.CharacterInSelection(posLineStart + ll->numCharsBeforeEOL - 1) != 0);
	if (!textSelected) {
		rcSegment.right = WxdColumnRight(vsDraw, rcLine);
		WxdSelectionRectangle(surface, model, vsDraw, rcSegment, fill, alpha);
	}
}

// The block of the selected text of a row: when the break of its line is selected too (and this is
// the last row of the line), it goes on over the break to the right edge of the column.
static void WxdSelectionText(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll,
	Sci::Line line, int subLine, Sci::Position posLineStart, Range lineRange, const SelectionSegment &portion,
	PRectangle rcSegment, PRectangle rcLine, ColourDesired fill, int alpha) {
	if (WxdSelectionShaped(model) && subLine == (ll->lines - 1) && line < model.pdoc->LinesTotal() - 1 &&
		portion.end.Position() == posLineStart + lineRange.end && !portion.end.VirtualSpace() &&
		model.sel.InSelectionForEOL(model.pdoc->LineStart(line + 1))) {
		const XYPOSITION right = WxdColumnRight(vsDraw, rcLine);
		if (right > rcSegment.right) {
			rcSegment.right = right;
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
    set(to4   [=[WxdEolSelection(surface, model, vsDraw, ll, model.pdoc->LineStart(line), rcLine, rcSegment, SelectionBackground(vsDraw, eolInSelection == 1, model.primarySelection), alpha);]=])
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
