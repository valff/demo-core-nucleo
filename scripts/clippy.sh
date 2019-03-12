#!/bin/bash

source $(dirname $0)/_env.sh
export RUSTC_WRAPPER=$(dirname $0)/_clippy_wrapper.sh

set -x
cargo check --target $BUILD_TARGET
