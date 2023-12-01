This project is a WIP for using a Raspberry Pi Zero W in USB Gadget mode to provide USB controllers for the Nintendo Switch.

# Configuring your Pi Zero W

You'll need to edit some boot settings on your SD card to load the appropriate drivers to use it as a USB Gadget. These are taken from: https://learn.adafruit.com/turning-your-raspberry-pi-zero-into-a-usb-gadget/serial-gadget

Edit config.txt, add:

```
dtoverlay=dwc2
```

Edit cmdline.txt, after rootwait add:

```
modules-load=dwc2,g_hid
```

# Build prerequisites

This project requires a recent version of the Rust compiler, as well as [`cross`](https://github.com/cross-rs/cross/wiki/Getting-Started) for cross-compiling to the Pi.

# Building

Run the build through `cross` like so:

```
cross build --target=arm-unknown-linux-gnueabi
```

There is also a `build.sh` script in the repository that combines the build step and using `scp` to transfer the resulting binary to a Pi over the network.

# Running

Connect your Pi Zero W to your Nintendo Switch using a USB Micro-A to USB-C cable (or a standard USB Micro-A cable through a USB-A-to-C adapter). Run the `pizero-gadget-gamepads` binary. Pair a bluetooth gamepad to the Pi (this step is currently manual, see `notes.txt` for some poorly-organized notes). Go to the Controllers screen on your Switch and press buttons on the gamepad and it should show up as a USB device.

The code as written should support mapping up to 4 simultaneous controllers.

# Troubleshooting

This works in theory but my own testing has shown some issues. I haven't been able to determine if it's a hardware issue with my Pi Zero or something that has changed in the Switch firmware since the last time I attempted this (but unfortunately lost the code I had written). YMMV

Controllers are mapped to the Switch controller layout using SDL2 mappings from [the SDL_GameControllerDB project](https://github.com/gabomdq/SDL_GameControllerDB). There isn't currently support for easily adding your own mappings, but you can add them to the `extra-mappings.txt` file in the repository and re-build the binary to have them included.

The binary has logging enabled at the `info` level by default. You can enable more verbose logs by setting `RUST_LOG=debug` in the environment before running it.
