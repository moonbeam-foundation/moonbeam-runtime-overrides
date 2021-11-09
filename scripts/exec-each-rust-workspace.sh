#!/bin/bash

INTEGER_REGEX='^[0-9]+$'

cd tracing

for D in *; do
    if [ -d "${D}" ]; then
      if [[ "${D}" =~ $INTEGER_REGEX ]] ; then
        echo "Exec command '$1' on tracing/${D}"
        cd ${D}
        $1 || exit 1
        cd ..
      fi
    fi
done
