#!/bin/bash

apt-get update -y 
apt-get upgrade -y 
apt install build-essential -y
apt install pkg-config gcc libssl-dev python3.8-venv -y

curl -sL https://deb.nodesource.com/setup_18.x | sudo -E bash - && sudo apt-get install -y nodejs && sudo npm install pm2 -g 

if [ ! -f $HOME/.cargo/env ] ;then
    chmod +x ./rust.sh && ./rust.sh -y
fi

source $HOME/.cargo/env
source $HOME/.bashrc

cd
if [ ! -d ~/miniconda3 ];then
    mkdir -p ~/miniconda3
    wget https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh -O ~/miniconda3/miniconda.sh
    bash ~/miniconda3/miniconda.sh -b -u -p ~/miniconda3
    rm -rf ~/miniconda3/miniconda.sh
    ~/miniconda3/bin/conda init bash
    source $HOME/.bashrc
    conda create -n nimble python=3.11 -y
fi


conda activate nimble

if [ -d nimble-miner-public ];then
    git clone https://github.com/nimble-technology/nimble-miner-public.git
    cd nimble-miner-public
    sed  sed -ir 's/numpy==1.26.4/numpy==1.24.4/' requirements.txt
    make install
fi

source ./nimenv_localminers/bin/activate

cd $HOME/clore
source $HOME/.cargo/env
cargo  build -r --bin monitor
# 系统初始化时，会运行以下此命令
# 下面是安装时就会自动运行
# apt install git -y
# mkdir -p clore/log
# git clone  https://github.com/zlseqx/clore.git temp >> $HOME/clore/log/server.txt 2>&1
# mv temp/* clore && rm -rf temp
# cd $HOME/clore && chmod +x env.sh rust.sh run.sh && ./env.sh >> $HOME/clore/log/server.txt 2>&1
# source $HOME/.cargo/env
# cargo run -r --bin monitor >>  $HOME/clore/monitor.txt 2>&1 &



