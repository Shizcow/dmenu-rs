#! /bin/sh

if [ "$depcheck" != "false" ]; then
   printf "Checking for xdg-open... "
   if command -v xdg-open &> /dev/null
   then
       echo "yes"
   else
       echo "no"
       >&2 echo "xdg-open required for plugin 'lookup' but not available. Install xdg or run make depcheck=false to continue anyway"
       exit 1
   fi
fi
