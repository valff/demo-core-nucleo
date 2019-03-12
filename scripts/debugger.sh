#!/bin/bash

source $(dirname $0)/_env.sh
trap 'kill $(jobs -p)' SIGINT SIGTERM EXIT

openocd $OPENOCD_CONFIG &
gdb-multiarch $RELEASE_ELF \
  -ex "target remote :3333" \
  -ex "monitor reset halt"
