#!/bin/bash

set -e
# IP=10.54.87.192
IP=192.168.1.9
USER=root
PASSWORD=root

cargo build --release --target arm-unknown-linux-gnueabihf
sshpass -p $PASSWORD ssh $USER@$IP 'systemctl stop ui' || true
sshpass -p $PASSWORD ssh $USER@$IP 'killall ui' || true
sshpass -p $PASSWORD scp ./target/arm-unknown-linux-gnueabihf/release/ui $USER@$IP:/root
cp ./target/arm-unknown-linux-gnueabihf/release/ui ../scripts/

# cd ../scripts/j2me/freej2me/
# ant clean || true
# ant
# sshpass -p $PASSWORD scp ./build/freej2me-sdl.jar $USER@$IP:/root/scripts/j2me/
# cp ./build/freej2me-sdl.jar ../freej2me-sdl.jar

sshpass -p $PASSWORD ssh $USER@$IP /root/ui
