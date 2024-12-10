#!/usr/bin/bash
rmmod uvcvideo
modprobe uvcvideo nodrop=5 timeout=10000 quirks=0x80
