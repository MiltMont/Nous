#!/usr/bin/env bash

cargo run -- -f $1 emit-code >test.s

gcc test.s -o test

./test

echo $?

# Removing files
rm test.s test
