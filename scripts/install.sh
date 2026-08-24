#!/bin/sh
set -e

log() {
    echo "\033[32m" "$@" "\033[0m"
}

log "Fix apt"
cat >/etc/apt/sources.list <<'EOF'
deb http://ports.ubuntu.com/ubuntu-ports focal main restricted universe multiverse
deb http://ports.ubuntu.com/ubuntu-ports focal-updates main restricted universe multiverse
deb http://ports.ubuntu.com/ubuntu-ports focal-backports main restricted universe multiverse
deb http://ports.ubuntu.com/ubuntu-ports focal-security main restricted universe multiverse

deb-src http://ports.ubuntu.com/ubuntu-ports focal main restricted universe multiverse
deb-src http://ports.ubuntu.com/ubuntu-ports focal-updates main restricted universe multiverse
deb-src http://ports.ubuntu.com/ubuntu-ports focal-backports main restricted universe multiverse
deb-src http://ports.ubuntu.com/ubuntu-ports focal-security main restricted universe multiverse
EOF

log "Installing display"
dtc -@ -I dts -O dtb -o sun8i-h3-spi-tft.dtbo sun8i-h3-spi-tft-fbtft.dts
cp sun8i-h3-spi-tft.dtbo /boot/dtb/overlay/
cp sun8i-h3-spi-tft.dtbo /boot/dtb-5.4.65-sunxi/overlay/

log "Installing keyboard"
dtc -@ -I dts -O dtb -o sun8i-h3-gpio-keyboard.dtbo sun8i-h3-gpio-keyboard.dts
cp sun8i-h3-gpio-keyboard.dtbo /boot/dtb/overlay/
cp sun8i-h3-gpio-keyboard.dtbo /boot/dtb-5.4.65-sunxi/overlay/

log "Remove graphical interface"
systemctl stop lightdm
systemctl disable lightdm
systemctl set-default multi-user.target
apt-get purge -y 'gnome-*' 'xserver-xorg*' 'lightdm*' 'x11-*'
apt-get autoremove --purge -y

log "Installing UI"
cp ui.service /etc/systemd/system/
cp ui /root/
systemctl daemon-reload
systemctl enable ui.service
systemctl disable getty@tty1.service

log "Update Env file"
sed -ie 's/overlays=usbhost2 usbhost3/overlays=spi-tft gpio-keyboard i2c0 usbhost2 usbhost3/' /boot/pbsbc01h3Env.txt

log "Create Env file backup"
cp /boot/pbsbc01h3Env.txt /boot/pbsbc01h3Env.txt.bk

log "Change host name"
echo "pda" > /etc/hostname

log "Change password"
echo "root:root" | chpasswd

log "Installing I2C tools"
apt install i2c-tools

log "----------------------------------------"
log "-                 Done                 -"
log "----------------------------------------"

# echo ads7846 | tee /etc/modules-load.d/ads7846.conf
# reboot
# export DISPLAY=:0
# export XAUTHORITY=/var/run/lightdm/root/:0
# firefox
# mpv --fs --video-rotate=90 --panscan=1.0 "$(yt-dlp -g "https://rutube.ru/video/xxx/" | head -n1)"
