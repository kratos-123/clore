import time
import pandas as pd 
import requests


def check_particle(addr):
    """This function inits the particle."""
    particle_url = "https://mainnet.nimble.technology/register_particle"
    res = requests.post(particle_url, timeout=60, json={"address": addr})
    # print(res.status_code, res.json())
    if res.status_code != 200:
        return False
    return True

def check_balance(addr):
    balance_url = "https://mainnet.nimble.technology/check_balance"
    data = {"address": addr}
    res = requests.post(balance_url, json=data)
    # print(res.status_code, res.json())
    if res.status_code == 200:
      if "Error" in res.json().get("msg"):
        return False
      else:
        return True
    else:
      return False


with open("/content/drive/MyDrive/Colab Notebooks/1.txt", "r") as f:
    addrs = f.readlines()

df = pd.DataFrame(columns=['地址', '类型'])


# addrs = ["nimble1zxf3s4cjnc6gn9h4tz4wg75l8qdn0xhy7vfwtt", "nimble1rfl7c637zqk49mtcnldlljehl5ru0ml80uf9nd"]

for addr in addrs:
    print("正在查询地址：", addr)
    if check_balance(addr):
        df.loc[len(df)] = [addr, "主钱包"]
        print("主钱包地址")
    elif check_particle(addr):
        df.loc[len(df)] = [addr, "子钱包"]
        print("子钱包地址")
    else:
        df.loc[len(df)] = [addr, "未注册钱包"]
        print("未注册钱包")
    time.sleep(1)

df.to_csv("/content/drive/MyDrive/Colab Notebooks/res.csv", index=False)

