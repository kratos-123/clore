#!/usr/bin/env bash
cd nimble-miner-public/


# python3 execute.py nimble1a67mj08trt4sxd4erhzrzgqnufma0h0khtdya3
# python3 execute.py $1 >> $HOME/clore/log/$1.txt 2>&1
# ./run.sh nimble1a67mj08trt4sxd4erhzrzgqnufma0h0khtdya3
# pm2 start "CUDA_VISIBLE_DEVICES=0 make run addr=$1" --name nimble --log $HOME/clore/log/$1.txt
conda init
conda activate nimble
source ./nimenv_localminers/bin/activate
pm2 start "CUDA_VISIBLE_DEVICES=0 make run addr=$1" --name nimble --log $HOME/clore/log/$1.txt
