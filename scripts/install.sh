#!/bin/bash
if ! cat /etc/os-release|grep Debian;then
    echo "Only support Debian"
    exit 1
fi

if ! command -v ipmitool &>/dev/null;then
    apt update
    apt install -y ipmitool
fi

if ! command -v python3 &>/dev/null;then
    apt install -y python3
fi

## get current path
current_path=$(cd `dirname $0`;pwd)

service_file="/etc/systemd/system/smartfan.service"
cat > $service_file <<EOF
[Unit]
Description=Lenovo HR650X Server Fan Control
After=network.target
Requires=network.target

[Service]
ExecStart=/usr/bin/python3 $current_path/hr650x_auto_fan.py in-band
WorkingDirectory=$current_path
Restart=always
RestartSec=5
User=root

[Install]
WantedBy=multi-user.target
EOF

chmod 644 $service_file
systemctl enable smartfan
systemctl start smartfan