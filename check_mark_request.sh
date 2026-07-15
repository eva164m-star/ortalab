#!/bin/bash

if [[ ! -f mark_request.txt ]]; then
    echo "You did not submit a mark_request.txt !!!"
    exit 1
fi
