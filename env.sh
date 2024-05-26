#!/bin/bash

# 项目依赖
apt install pkg-config gcc libssl-dev python3.8-venv -y

#rust安装
if [ ! -f $HOME/.cargo/env ] ;then
    chmod +x ./rust.sh && ./rust.sh -y
fi

if [ ! -d "nimble-miner-public" ]; then
    git clone https://github.com/nimble-technology/nimble-miner-public.git;
    sed -ir 's/print\((.*)\)/print(\1,flush=True)/' nimble-miner-public/execute.py
    sed -ir 's/"\\0.*"/""/' nimble-miner-public/execute.py
fi

cd nimble-miner-public

python3 -m venv ./venv
source ./venv/bin/activate

python3 -m pip install --upgrade pip
python3 -m pip install requests==2.31.0 
python3 -m pip install torch==2.2.1
python3 -m pip install accelerate==0.27.0 
python3 -m pip install transformers==4.38.1 
python3 -m pip install datasets==2.17.1 
python3 -m pip install numpy
python3 -m pip install gitpython==3.1.42 
python3 -m pip install prettytable==3.10.0


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



