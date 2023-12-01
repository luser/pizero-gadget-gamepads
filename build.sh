#!/bin/sh

set -e

cross build --target=arm-unknown-linux-gnueabi
scp target/arm-unknown-linux-gnueabi/debug/pizero-gadget-gamepads pi@raspberrypi.local:
