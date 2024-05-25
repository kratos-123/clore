#!/usr/bin/env bash

source ./nimble-miner-public/venv/bin/activate

mkdir -p log
# python3 execute.py nimble19ds02xkxwfw9l2k8jdlx9ns7s5p0aguxd0v75c
# ./run.sh nimble19ds02xkxwfw9l2k8jdlx9ns7s5p0aguxd0v75c >> log.txt 2>&1 &
python3 ./nimble-miner-public/execute.py $1 >> log/$1.txt 2>&1