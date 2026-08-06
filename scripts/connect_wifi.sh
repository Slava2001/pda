#!/bin/bash

set -e

if [ $# -ne 3 ]; then
    echo "Usage: $0 <SSID> <PASSWORD> <INTERFACE>"
    exit 1
fi

SSID="$1"
PASSWORD="$2"
INTERFACE="$3"

nmcli device wifi connect $SSID password $PASSWORD ifname $INTERFACE
ifconfig wlan0 | grep "inet "
