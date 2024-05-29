#!/bin/bash
cd $HOME

apt install build-essential  pkg-config gcc libssl-dev python3.8-venv -y
curl -sL https://deb.nodesource.com/setup_18.x | sudo -E bash - && sudo apt-get install -y nodejs && sudo npm install pm2 -g 

if [ ! -d ~/miniconda3 ];then
    mkdir -p ~/miniconda3
    wget https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh -O ~/miniconda3/miniconda.sh
    bash ~/miniconda3/miniconda.sh -b -u -p ~/miniconda3
    echo "export PATH=$HOME/miniconda3/bin:$PATH" >> $HOME/.bashrc
    # rm -rf ~/miniconda3/miniconda.sh
fi

source $HOME/.bashrc
conda init
nimble=`conda info -e | grep nimble`;
if [[ "$nimble" == "" ]];then
    conda create -n nimble python=3.11 -y    
fi

conda activate nimble

# clore目录相关的操作
cd $HOME/clore

# 安装rust
if [ ! -f $HOME/.cargo/env ] ;then
    chmod +x ./rust.sh && ./rust.sh -y
fi


#安装挖矿程序
if [ ! -d nimble-miner-public ];then
    git clone https://github.com/nimble-technology/nimble-miner-public.git
    cd nimble-miner-public
    sed  sed -ir 's/numpy==1.26.4/numpy==1.24.4/' requirements.txt
    make install
fi


cd $HOME/clore
source $HOME/.cargo/env
cargo  build -r --bin monitor
# 系统初始化时，会运行以下此命令
# 下面是安装时就会自动运行
# cd $HOME
# apt update -y 
# apt install git -y
# echo "export SERVER_ID={server_id}" >> $HOME/.bashrc
# echo "export NVIDIA_CARD_NUMBER={card_number}" >> $HOME/.bashrc
# source $HOME/.bashrc
# git clone  https://github.com/zlseqx/clore.git >> $HOME/server.txt 2>&1

# cd $HOME/clore
# chmod +x env.sh rust.sh run.sh && ./env.sh >> $HOME/server.txt 2>&1

# # 防止内部被cd,需要切换到clore目录操作
# cd $HOME/clore
# source $HOME/.cargo/env
# source $HOME/.bashrc

# # 激活pm2环境,用pm2管理cargo 监控进程
# conda init
# conda activate nimble
# pm2 start "cargo run -r --bin monitor" --name monitor --logs $HOME/clore/monitor.txt


