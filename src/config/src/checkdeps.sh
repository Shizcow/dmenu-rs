#! /bin/sh

FAILED=0

# we can assume sh is installed or else we wouldn't be here

printf "Checking for $CC... "
if command -v $CC &> /dev/null
then
    echo "yes"
else
    echo "no"
    >&2 echo "Build-time dependency $CC not installed. Install it or change the C compiler used in config.mk"
    FAILED=1
fi

printf "Checking for X11 headers... "
if pkg-config --exists x11
then
    echo "yes"
else
    echo "no"
	>&2 echo "Build-time dependency X11 (required for Xlib.h) is not present. Install the xorg development packages"
    FAILED=1
fi

printf "Checking for fontconfig headers... "
if pkg-config --exists fontconfig
then
    echo "yes"
else
    echo "no"
	>&2 echo "Build-time dependency fontconfig is not present. Install fontconfig packages"
    FAILED=1
fi

if [ "$XINERAMA" = "true" ]; then
    printf "Checking for xinerama headers... "
    if pkg-config --exists xinerama
    then
	echo "yes"
    else
	echo "no"
	>&2 echo "Build-time dependency Xinerama is not present. Install xinerama package(s) or disable the feature in config.mk"
	FAILED=1
    fi
fi

if [ $FAILED != 0 ]; then
    exit 1
fi
