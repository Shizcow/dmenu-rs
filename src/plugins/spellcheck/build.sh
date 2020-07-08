#! /bin/sh

# check that aspell is installed
printf "Checking for aspell... "
if [[ -x zenity ]]
then
    echo "yes"
else
    echo "no"
    echo "aspell not found! Install aspell to use the spellcheck plugin"
    exit 1
fi
