#!/bin/bash

# $1 contains the composed message
# $2 contains the path to the peers.txt file, not relevant for this script
# $3 contains the loop iteration number

# make sure to change the "user" variable to the actual username or user ID
# of the user you want to send the notification to, e.g. 1000, "bob" or "alice".

user=1000

icon="network-wireless-disconnected"
urgency="critical"
loop_number="$3"
message="$1"

if [[ $loop_number = 0 ]]; then
    # run 0
    summary="WireGuard Monitor: first run"
else
    summary="WireGuard Monitor: update"
fi

systemd-run --machine=${user}@.host --user \
    notify-send \
        --icon="$icon" \
        --urgency="$urgency" \
        "$summary" \
        "$message"
