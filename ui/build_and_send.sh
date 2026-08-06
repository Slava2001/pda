#!/bin/bash

set -e
cargo build --release --target arm-unknown-linux-gnueabihf
sshpass -p root ssh root@192.168.1.24 'systemctl stop ui' || true
sshpass -p root ssh root@192.168.1.24 'killall ui' || true
sshpass -p root scp ./target/arm-unknown-linux-gnueabihf/release/ui root@192.168.1.24:/root
cp ./target/arm-unknown-linux-gnueabihf/release/ui ../scripts/
sshpass -p root ssh root@192.168.1.24 /root/ui
