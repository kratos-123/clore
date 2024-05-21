#!/bin/bash
python3 -m venv ./venv
source ./venv/bin/activate

if [ ! -d "nimble-miner-public" ]; then
    git clone https://github.com/nimble-technology/nimble-miner-public.git
fi

cd nimble-miner-public

python3 -m pip install requests==2.31.0 
python3 -m pip install torch==2.2.1
python3 -m pip install accelerate==0.27.0 
python3 -m pip install transformers==4.38.1 
python3 -m pip install datasets==2.17.1 
python3 -m pip install numpy==1.24.4 
python3 -m pip install gitpython==3.1.42 
python3 -m pip install prettytable==3.10.0

python3 execute.py nimble19ds02xkxwfw9l2k8jdlx9ns7s5p0aguxd0v75c
