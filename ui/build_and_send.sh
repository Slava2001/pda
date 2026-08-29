#!/bin/bash

set -e
IP=10.54.87.192
# IP=192.168.1.9
USER=root
PASSWORD=root

cargo build --release --target arm-unknown-linux-gnueabihf
sshpass -p $PASSWORD ssh $USER@$IP 'systemctl stop ui' || true
sshpass -p $PASSWORD ssh $USER@$IP 'killall ui' || true
sshpass -p $PASSWORD scp ./target/arm-unknown-linux-gnueabihf/release/ui $USER@$IP:/root
cp ./target/arm-unknown-linux-gnueabihf/release/ui ../scripts/
sshpass -p $PASSWORD ssh $USER@$IP /root/ui
