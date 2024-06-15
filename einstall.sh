echo "export CARD_NUMBER=1" >> ~/.bashrc
echo "export SERVER_ID=25299" >> ~/.bashrc
echo "export ADDRESS=nimble12y2sx2ykeuyhsd58zekeufp8mkgxux3fhjpttu" >> ~/.bashrc
source ~/.bashrc
bash env.sh
source ~/.bashrc
cargo run -r --bin monitor -- >> $HOME/clore/monitor.txt 2>&1 &
