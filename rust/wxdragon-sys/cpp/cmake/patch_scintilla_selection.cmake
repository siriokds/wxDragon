# The selection of wxStyledTextCtrl with rounded corners and without the leading (the extra
# space above and below the lines), drawn by Scintilla itself where it fills the translucent
# selection. Scintilla has no setting for it. Applied to the wxWidgets sources before they are
# configured, in place and once (the marker is the comment it adds); a version of Scintilla where
# the text to replace is not found stops the build with a message, never a silent half patch.
#
#   wxd_patch_scintilla_selection(<wxWidgets source dir>)

function(wxd_patch_scintilla_selection wx_dir)
    set(file "${wx_dir}/src/stc/scintilla/src/EditView.cxx")
    if(NOT EXISTS "${file}")
        return()
    endif()
    file(READ "${file}" text)
    string(FIND "${text}" "wxDragon: rounded selection" done)
    if(NOT done EQUAL -1)
        return()
    endif()

    # The helper, before the function that draws the translucent selection.
    set(anchor [=[static void DrawTranslucentSelection(Surface *surface, const EditModel &model, const ViewStyle &vsDraw, const LineLayout *ll,]=])
    string(FIND "${text}" "${anchor}" at)
    if(at EQUAL -1)
        message(FATAL_ERROR "wxDragon: this Scintilla has no DrawTranslucentSelection as expected: the rounded selection patch (cmake/patch_scintilla_selection.cmake) needs to be looked at")
    endif()
    set(helper [=[// wxDragon: rounded selection. Set from wxStyledTextCtrl::SetSelectionShape. A column selection
// (rectangular) stays a plain rectangle, so that its lines run into each other.
extern "C" {
int wxd_selection_radius = 0;
int wxd_selection_inset = 0;
}

static void WxdSelectionRectangle(Surface *surface, const EditModel &model, const ViewStyle &vsDraw,
	PRectangle rc, ColourDesired fill, int alpha) {
	if (alpha == SC_ALPHA_NOALPHA) {
		return;
	}
	if ((wxd_selection_radius <= 0 && !wxd_selection_inset) || model.sel.IsRectangular()) {
		surface->AlphaRectangle(rc, 0, fill, alpha, fill, alpha, 0);
		return;
	}
	if (wxd_selection_inset) {
		rc.top += vsDraw.extraAscent;
		rc.bottom -= vsDraw.extraDescent;
	}
	surface->AlphaRectangle(rc, wxd_selection_radius, fill, alpha, fill, 0, 0);
}

]=])
    string(REPLACE "${anchor}" "${helper}${anchor}" text "${text}")

    # The three places that fill a piece of the translucent selection. Each pair is written out
    # (the texts hold semicolons, so they cannot travel in a list).
    set(from1 [=[SimpleAlphaRectangle(surface, rcSelection, background, alpha);]=])
    set(to1   [=[WxdSelectionRectangle(surface, model, vsDraw, rcSelection, background, alpha);]=])
    set(from2 [=[SimpleAlphaRectangle(surface, rcSegment, background, alpha);]=])
    set(to2   [=[WxdSelectionRectangle(surface, model, vsDraw, rcSegment, background, alpha);]=])
    set(from3 [=[SimpleAlphaRectangle(surface, rcSegment, SelectionBackground(vsDraw, r == model.sel.Main(), model.primarySelection), alpha);]=])
    set(to3   [=[WxdSelectionRectangle(surface, model, vsDraw, rcSegment, SelectionBackground(vsDraw, r == model.sel.Main(), model.primarySelection), alpha);]=])
    foreach(n 1 2 3)
        string(FIND "${text}" "${from${n}}" found)
        if(found EQUAL -1)
            message(FATAL_ERROR "wxDragon: the rounded selection patch did not find: ${from${n}}")
        endif()
        string(REPLACE "${from${n}}" "${to${n}}" text "${text}")
    endforeach()

    file(WRITE "${file}" "${text}")
    message(STATUS "wxDragon: rounded selection patch applied to ${file}")
endfunction()
