#!/bin/bash

cargo build
sudo RUST_BACKTRACE=1 cargo test &
sleep 3
sudo ifconfig utun5 10.25.0.1/24 10.25.0.2 netmask 255.255.255.255 && sudo route add 10.25/24 10.25.0.1
ping -c 1 -W 1 10.25.0.100
