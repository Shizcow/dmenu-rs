#! /bin/sh

# check that aspell is installed
printf "Checking for aspell... "
if command -v aspell &> /dev/null
then
    echo "yes"
else
    echo "no"
    echo "aspell not found! Install aspell to use the spellcheck plugin"
    exit 1
fi
