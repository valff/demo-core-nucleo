BUILD_TARGET=thumbv7em-none-eabihf
TEST_TARGET=thumbv7em-linux-eabihf
OPENOCD_CONFIG="-f board/stm32l4discovery.cfg"
ITM_FREQ=80000000
ITM_FIFO=/tmp/drone-itm.fifo

ELF_NAME=$(basename $(pwd))
RELEASE_ELF="target/$BUILD_TARGET/release/$ELF_NAME"
