#!/bin/bash
cd /home/douglas/code/game3
timeout 5 cargo run --release 2>&1 | grep -E "(GPU Grid|Switching|terrain)"