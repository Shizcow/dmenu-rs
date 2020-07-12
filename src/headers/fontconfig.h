#include <fontconfig/fontconfig.h>

#define __redef_tmp FC_COLOR
#undef FC_COLOR
#define FC_COLOR (__redef_tmp "\0")
#undef __redef_tmp
