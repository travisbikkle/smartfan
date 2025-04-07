#!/bin/bash
script_dir=$(cd `dirname "$0"`; pwd)
echo "Script is located in $script_dir"
if ! command -v screen;then
  apt install -y screen
fi

cd $script_dir
chmod +x smartfan

screen_name="smartfan"
cmd="$script_dir/smartfan"

screen -dmS $screen_name
screen -x -S $screen_name -p 0 -X stuff "$cmd"  # 发送命令到screen会话
screen -x -S $screen_name -p 0 -X stuff '\n'    # 模拟回车执行