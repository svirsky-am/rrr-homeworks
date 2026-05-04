#!/bin/bash

set -ex

CUR_DIR=$(dirname $0)
echo CUR_DIR is $CUR_DIR
cd $CUR_DIR
wasm-pack build --target web
