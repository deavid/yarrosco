#!/bin/bash

# Dependency:
# $ sudo apt install playerctl

while :
do
	# playerctl metadata --format "Now playing: {{ artist }} - {{ album }} - {{ title }}               " > .currentlyplaying.txt
	# playerctl metadata --format "Now playing: {{ artist }} - {{ title }}               " > .currentlyplaying.txt
    playerctl metadata --all-players --format '{{ status }}: {{ artist }} - {{ title }}               ' | grep Playing | tr '\n' ' ' > .currentlyplaying.txt
    # copying prevents flickering in OBS (Bash first creates an empty file)
    cp .currentlyplaying.txt currentlyplaying.txt
	sleep 1
done
