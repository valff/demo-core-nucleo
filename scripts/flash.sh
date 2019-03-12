#!/bin/bash

source $(dirname $0)/_env.sh
export RUSTC_WRAPPER=$(dirname $0)/_rustc_wrapper.sh

cargo build --target $BUILD_TARGET --release && \
openocd $OPENOCD_CONFIG \
  -c "init" \
  -c "reset halt" \
  -c "flash write_image erase $RELEASE_ELF 0 elf" \
  -c "verify_image $RELEASE_ELF 0 elf" \
  -c "reset run" \
  -c "shutdown"
