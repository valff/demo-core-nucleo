#!/bin/bash

source $(dirname $0)/_env.sh
trap 'kill $(jobs -p)' SIGINT SIGTERM EXIT

rm -f $ITM_FIFO
mkfifo -m 0644 $ITM_FIFO
openocd $OPENOCD_CONFIG \
  -c "itm ports on" \
  -c "tpiu config internal $ITM_FIFO uart off $ITM_FREQ" &
itmdump -f $ITM_FIFO
