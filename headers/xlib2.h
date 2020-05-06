#include <X11/Xlib.h>

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
