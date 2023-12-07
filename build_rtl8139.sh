#!/bin/sh
set -eu
(cd samples/net_rtl8139/ && cargo build)
./tools/create_so/create_so_rust.sh samples/net_rtl8139/target/debug/libudi_net_rtl8139.a