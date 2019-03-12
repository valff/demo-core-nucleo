#!/bin/bash

source $(dirname $0)/_env.sh
export RUSTC_WRAPPER=$(dirname $0)/_rustc_wrapper.sh

cargo build --target $BUILD_TARGET --release && \
cargo objdump --target $BUILD_TARGET --release --bin $ELF_NAME -- \
  -disassemble -demangle -s -all-headers | \
pager
