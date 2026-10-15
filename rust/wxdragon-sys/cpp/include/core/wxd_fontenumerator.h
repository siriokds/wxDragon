#ifndef WXD_FONTENUMERATOR_H
#define WXD_FONTENUMERATOR_H

#include "../wxd_types.h"

#ifdef __cplusplus
extern "C" {
#endif

// Returns the face names of the fonts installed on the system (only the fixed
// width ones if fixed_width_only), as a newly allocated wxd_ArrayString_t*
// owned by the caller (free with wxd_ArrayString_Free).
WXD_EXPORTED wxd_ArrayString_t*
wxd_FontEnumerator_GetFacenames(bool fixed_width_only);

// Whether a font with this face name is installed. Unlike creating a font,
// which silently falls back to another face, this tells a misspelled name.
WXD_EXPORTED bool
wxd_FontEnumerator_IsValidFacename(const char* facename);

#ifdef __cplusplus
}
#endif

#endif // WXD_FONTENUMERATOR_H
