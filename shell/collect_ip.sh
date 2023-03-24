#!/bin/bash

while true
do
        /usr/bin/awk -F, '/^CLIENT_LIST/&&$2 ~ /^[0-9]+$/{printf "SET "$2" "$4" EX 30\n"}' /run/openvpn-server/status-server.log |/usr/bin/redis-cli -p 6379 --pipe
        sleep 30
done
