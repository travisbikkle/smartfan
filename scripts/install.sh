#!/bin/bash
if ! cat /etc/os-release|grep Debian;then
    echo "Only support Debian"
    exit 1
fi

apt install -y ipmitool screen


## get current path
current_path=$(cd `dirname $0`;pwd)
chmod +x $current_path/smartfan

sed "s/out-band/in-band/g" -i "${current_path}/config.yaml"

service_file="/etc/systemd/system/smartfan.service"
cat > $service_file <<EOF
[Unit]
Description=Lenovo HR650X Server Fan Control
After=network.target
Requires=network.target

[Service]
ExecStart=$current_path/start_with_screen.sh # 没有用
WorkingDirectory=$current_path
Restart=always
Fork=yes
RestartSec=5
User=root

[Install]
WantedBy=multi-user.target
EOF

chmod 644 $service_file
systemctl enable smartfan
systemctl daemon-reload
systemctl restart smartfan