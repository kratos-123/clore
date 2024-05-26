#!/usr/bin/env bash
cd nimble-miner-public/
source venv/bin/activate

# python3 execute.py nimble1a67mj08trt4sxd4erhzrzgqnufma0h0khtdya3
# ./run.sh  >> log.txt 2>&1 &

python3 execute.py $1 >> $HOME/clore/log/$1.txt 2>&1