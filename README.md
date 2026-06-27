Very experimental bindings to allow writing UDI (Universal Driver Interface) drivers in rust

Uses an absolute pile of hacks to make the callback-based async of UDI work with rust's poll-based async

DO NOT USE THIS. It's mostly a fun experiment to see if the concept can work and be anywhere near efficient.

# Examples
- UDI environment (with crude simulations of a RealTek 8029, 8139, and a standard PC serial port)

# Usage
## Basic test
`cargo run --bin udi-environment <path_to_driver_so>`

Runs the sample environment with the passed driver. If no driver is provided, it runs the built-in NE2000 (RealTek 8029) driver.

## Build RTL8139 driver
`./build_rtl8139.sh` - Builds the driver, then links it into a valid UDI loadable object

## Use pre-compiled (Rust) UDI drivers
Run `tools/create_so/create_so_rust.sh` to convert a rust `.a` file into a loadable `.so` for the example environment

## Use pre-compiled (C) UDI drivers
Run `tools/create_so/create_so.sh` to convert a standard pre-compiled UDI module to a `.so`