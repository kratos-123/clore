#!/usr/bin/env bash
cd nimble-miner-public/


# python3 execute.py nimble1a67mj08trt4sxd4erhzrzgqnufma0h0khtdya3
# python3 execute.py $1 >> $HOME/clore/logs/$1.txt 2>&1
# ./run.sh nimble1a67mj08trt4sxd4erhzrzgqnufma0h0khtdya3
# pm2 start "CUDA_VISIBLE_DEVICES=0 make run addr=$1" --name nimble --log $HOME/clore/logs/$1.txt
nivdai_card_number=${1:-'0'}
# echo "CUDA_VISIBLE_DEVICES=$nivdai_card_number make run addr=$2";
conda init
conda activate nimble
source ./nimenv_localminers/bin/activate

pm2 start "CUDA_VISIBLE_DEVICES=${nivdai_card_number} make run addr=$2" --name nimble${nivdai_card_number} --log $HOME/clore/logs/$2.txt
