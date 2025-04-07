#!/bin/bash
pid=$(screen -ls|grep -Eo "[0-9]+\.smartfan"|head -n 1|cut -d . -f 1)

if [ -z "$pid" ]; then
    echo "No screen session found"
else
    echo "Screen session found: $pid"
    screen -r $pid
fi