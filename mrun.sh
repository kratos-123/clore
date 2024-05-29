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
mkdir -p ~/miniconda3
wget https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh -O ~/miniconda3/miniconda.sh
bash ~/miniconda3/miniconda.sh -b -u -p ~/miniconda3
rm -rf ~/miniconda3/miniconda.sh
~/miniconda3/bin/conda init bash
source $HOME/.bashrc



conda create -n nimble python=3.11 -y
conda activate nimble

mkdir $HOME/nimble && cd $HOME/nimble


cd  $HOME/nimble
git clone https://github.com/b5prolh/nimble-miner-public.git
cd nimble-miner-public
sed  sed -ir 's/numpy==1.26.4/numpy==1.24.4/' requirements.txt
make install
source ./nimenv_localminers/bin/activate