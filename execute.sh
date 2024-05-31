#!/usr/bin/env bash
cd nimble-miner-public/


# python3 execute.py nimble1a67mj08trt4sxd4erhzrzgqnufma0h0khtdya3
# python3 execute.py $1 >> $HOME/clore/logs/$1.txt 2>&1
# ./run.sh nimble1a67mj08trt4sxd4erhzrzgqnufma0h0khtdya3
# pm2 start "CUDA_VISIBLE_DEVICES=0 make run addr=$1" --name nimble --log $HOME/clore/logs/$1.txt
# echo "CUDA_VISIBLE_DEVICES=$nivdai_card_number make run addr=$2";

# 怎么执行
# execute.sh <restart|start> <service_name> <card_num> <addres> 
# source ~/.bashrc
# conda init
# conda activate nimble
# source ./nimenv_localminers/bin/activate

action=$1
service_name=$2
card_number=${3:-'0'}
address=$4

case $action in
    "start" ) 
        # 如果变量与 pattern1 匹配，则执行的代码
        ;; 
    "restart") 
        # 如果变量与 pattern2 匹配，则执行的代码
        ;; 
    * ) 
        # 如果变量与任何模式都不匹配，则执行的代码
        ;; 
esac

# pm2 start "CUDA_VISIBLE_DEVICES=${nivdai_card_number} make run addr=$2" --name nimble${nivdai_card_number} --log $HOME/clore/logs/$2.txt
