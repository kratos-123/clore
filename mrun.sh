#!/usr/bin/env bash
set -e

address=`env|grep ADDRESS |grep -v grep |awk -F '=' '{print $2}'`;
addr=(${address//,/ });
 
for var in ${addr[@]}
do
   echo $var
done

addr_length=${#addr[@]}
echo "ADDRESS:数组长度为: $addr_length"

nvidia=`nvidia-smi -L | awk -F ' ' '{print $2}'| sed 's/://'`;

nvidias=(${nvidia//\n/ })
for var in ${nvidias[@]}
do
   echo $var
done


nvidia_length=${#nvidias[@]}
echo "NVIDIA:数组长度为: $nvidia_length";

if [ $nvidia_length -ne $addr_length ];then
    echo "Error:当前显卡和地址环境数量不匹配"
    exit 1;
fi

#cd nimble-miner-public/
#
#conda init
#conda activate nimble
#source ./nimenv_localminers/bin/activate
#
#pm2 start "CUDA_VISIBLE_DEVICES=${nivdai_card_number} make run addr=$2" --name nimble${nivdai_card_number} --log $HOME/clore/log/$2.txt


for index in ${addr[@]}
do
   command="pm2 start "CUDA_VISIBLE_DEVICES=${index} make run addr=${addr[index]}" --name nimble${index} --log $HOME/clore/log/${addr[index]}.txt"
done


