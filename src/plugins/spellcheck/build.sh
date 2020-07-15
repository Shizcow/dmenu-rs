#! /bin/sh

if [ "$depcheck" != "false" ]; then
   printf "Checking for aspell... "
   if command -v aspell &> /dev/null
   then
       echo "yes"
   else
       echo "no"
       >&2 echo "Runtime dependency aspell missing. Install it or run make depcheck=false to continue anyway"
       exit 1
   fi

   printf "Checking for xclip... "
   if command -v xclip &> /dev/null
   then
       echo "yes"
   else
       echo "no"
       >&2 echo "Install xclip to use the spellcheck plugin. Install it or run make depcheck=false to continue anyway"
       exit 1
   fi
fi
