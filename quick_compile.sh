#!/usr/bin/env bash

cargo run -- --file $1 --emitcode >$2

gcc $2 -o $3

./$3

echo $?

# Removing files
rm $2 $3
