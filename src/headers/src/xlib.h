#include <X11/Xlib.h>

// The following just add null chars to their strings
// This is implicit in C, but bindgen doesn't include
// these characters. Doing this makes code much cleaner

#define __redef_tmp XNInputStyle
#undef XNInputStyle
#define XNInputStyle (__redef_tmp "\0")
#undef __redef_tmp

#define __redef_tmp XNClientWindow
#undef XNClientWindow
#define XNClientWindow (__redef_tmp "\0")
#undef __redef_tmp

#define __redef_tmp XNFocusWindow
#undef XNFocusWindow
#define XNFocusWindow (__redef_tmp "\0")
#undef __redef_tmp
