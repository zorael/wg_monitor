#!/bin/bash

# $1 contains the composed message
# $2 contains the path to the peers.txt file, not relevant for this script
# $3 contains the loop iteration number

icon="network-wireless-disconnected"
urgency="critical"
loop_number=$3
message="$1"

ids=( $(loginctl list-sessions -j | jq -r '.[] | .session') )

if [[ "$loop_number" = "0" ]]; then
    # run 0
    summary="WireGuard Monitor: first run"
else
    summary="WireGuard Monitor: update"
fi

for id in "${ids[@]}" ; do
    [[ $(loginctl show-session $id --property=Type) =~ (wayland|x11) ]] || continue

    user=$(loginctl show-session $id --property=Name --value)

    systemd-run --machine=${user}@.host --user \
        notify-send \
            --icon="$icon" \
            --urgency="$urgency" \
            "$summary" \
            "$message"
done
