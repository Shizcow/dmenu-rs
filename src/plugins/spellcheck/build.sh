#! /bin/sh

# check that aspell is installed
printf "Checking for aspell... "
if command -v aspell &> /dev/null
then
    echo "yes"
else
    echo "no"
    >&2 echo "Install aspell to use the spellcheck plugin"
    exit 1
fi
