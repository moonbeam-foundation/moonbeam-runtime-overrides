#!/bin/bash

cd tracing

for D in *; do
    if [ -d "${D}" ]; then
        echo "Exec command '$1' on tracing/${D}"
        cd ${D}
        $1
        cd ..
    fi
done
