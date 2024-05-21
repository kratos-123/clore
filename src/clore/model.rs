use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug,Clone)]
struct Net {
    up: f64,
    down: f64,
    cc: String,
}

#[derive(Serialize, Deserialize, Debug,Clone)]
struct Specs {
    mb: String,
    cpu: String,
    cpus: String,
    ram: f64,
    disk: String,
    disk_speed: f32,
    gpu: String,
    gpuram: f32,
    net: Net,
}

#[derive(Serialize, Deserialize, Debug,Clone)]
struct Price {
    on_demand: HashMap<String, f64>,
    spot: HashMap<String, f64>,
}

#[derive(Serialize, Deserialize, Debug,Clone)]
struct Server {
    allowed_coins: Vec<String>,
    id: u32,
    owner: u32,
    mrl: u32,
    price: Price,
    rented: bool,
    specs: Specs,
    rating: HashMap<String, f32>,
}

#[derive(Serialize, Deserialize, Debug,Clone)]
struct Marketplace {
    servers: Vec<Server>,
    my_servers: Vec<u32>,
    code: u32,
}

impl Deref for Marketplace {
    type Target = Vec<Server>;

    fn deref(&self) -> &Self::Target {
        &self.servers
    }
}

impl DerefMut for Marketplace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.servers
    }
}

enum CardType {
    NVIDIA4090,
    NVIDIA4080S,
    NVIDIA4080,
    NVIDIA4070,
    NVIDIA4070TI,
    NVIDIA3090,
    NVIDIA3080,
}


struct Card{
    server_id:u32,
    price_on_demand:f32,
    price_spot:f32,
    mrl:u32,
    card_number:u32,
    rented:bool,
    card_type:CardType,
}

#[cfg(test)]
mod Model {
    use regex::Regex;

    use crate::clore::model::Server;

    use super::Marketplace;
    use std::io::Read;

    #[test]
    fn marketplace_test() {
        let mut row = String::from(
            r#"{"servers":[{"id":1398,"owner":979,"mrl":1440,"price":{"on_demand":{"bitcoin":0.00029,"CLORE-Blockchain":60},"spot":{"bitcoin":0.00029,"CLORE-Blockchain":50}},"rented":false,"specs":{"mb":"B450 Steel Legend","cpu":"AMD Athlon 200GE with Radeon Vega Graphics","cpus":"2/4","ram":14.91,"disk":"RevuAhnx20900Gx20BIZx 97.5735GB","disk_speed":1754.3859649122808,"gpu":"2x NVIDIA GeForce RTX 3080","gpuram":10,"net":{"up":2.99,"down":4.78,"cc":"KR"},"backend_version":9,"pcie_rev":1,"pcie_width":1,"pl":[320,320]},"reliability":1,"allowed_coins":["bitcoin","CLORE-Blockchain"],"rating":{"avg":5,"cnt":2},"cuda_version":"12.0"},{"id":1397,"owner":979,"mrl":1440,"price":{"on_demand":{"bitcoin":0.00029,"CLORE-Blockchain":60},"spot":{"bitcoin":0.00029,"CLORE-Blockchain":50}},"rented":false,"specs":{"mb":"B450 Steel Legend","cpu":"AMD Athlon 200GE with Radeon Vega Graphics","cpus":"2/4","ram":14.91,"disk":"RevuAhnx20900Gx20BIZx 97.8746GB","disk_speed":1779.359430604982,"gpu":"2x NVIDIA GeForce RTX 3080","gpuram":10,"net":{"up":3.64,"down":1.71,"cc":"KR"},"backend_version":9,"pcie_rev":1,"pcie_width":1,"pl":[320,320]},"reliability":1,"allowed_coins":["bitcoin","CLORE-Blockchain"],"rating":{"avg":5,"cnt":4},"cuda_version":"12.0"},{"id":1393,"owner":929,"mrl":1440,"price":{"on_demand":{"bitcoin":0.00029,"CLORE-Blockchain":60},"spot":{"bitcoin":0.00029,"CLORE-Blockchain":50}},"rented":false,"specs":{"mb":"B450 Steel Legend","cpu":"AMD Athlon 200GE with Radeon Vega Graphics","cpus":"2/4","ram":14.91,"disk":"RevuAhnx20900Gx20BIZx 98.018GB","disk_speed":1742.1602787456447,"gpu":"2x NVIDIA GeForce RTX 3080","gpuram":10,"net":{"up":0.24,"down":3.32,"cc":"KR"},"backend_version":9,"pcie_rev":1,"pcie_width":1,"pl":[320,320]},"reliability":0.9999,"allowed_coins":["bitcoin","CLORE-Blockchain"],"rating":{"avg":5,"cnt":2},"cuda_version":"12.0"},{"id":1229,"owner":775,"mrl":1440,"price":{"on_demand":{"bitcoin":0.0025},"spot":{"bitcoin":0.00017698}},"rented":false,"specs":{"mb":"ROG STRIX Z490-E GAMING","cpu":"Intel(R) Celeron(R) G5905 CPU @ 3.50GHz","cpus":"2/2","ram":15.8085,"disk":" 62.9768GB","disk_speed":2252.252252252252,"gpu":"4x NVIDIA GeForce RTX 3090","gpuram":24,"net":{"up":59.81,"down":39.86,"cc":"KR"},"backend_version":8,"pcie_rev":1,"pcie_width":1,"pl":[350,350,350,350]},"reliability":1,"allowed_coins":["bitcoin"],"rating":{"avg":3.5,"cnt":6},"cuda_version":"12.0"}],"my_servers":[],"code":0}"#,
        );
        let result = serde_json::from_str::<Marketplace>(&row);
        assert_eq!(true, result.is_ok());
    }

    #[test]
    fn marketplace_filter_test() {
        let mut marketplace =
            std::fs::File::open("./market.json").expect("当前目录下./market.json文件不存在！");
        let mut row = String::from("");
        let _ = marketplace.read_to_string(&mut row);

        // println!("{:?}",row);
        let result = serde_json::from_str::<Marketplace>(&row);
        //  println!("{:?}",result);
        assert_eq!(true, result.is_ok());
        let model = result.unwrap();
        let regex = Regex::new(r"(3080|3090|4070|4080|4080|4090)").unwrap();
        let servers: Vec<Server> = (*model).iter().filter(|item| {
            let machineProperties = &item.specs;
            let gpu = &machineProperties.gpu;
            regex.is_match(&gpu) && item.rating.get("avg").unwrap() > &2f32 && item.allowed_coins.contains(&"CLORE-Blockchain".to_string())
        }).map(|itme|{
            let card_info = itme.specs.gpu.split(' ').map(|item|item.to_string()).collect::<Vec<String>>();
            let number = card_info.get(0).map_or(0, |s|{
                let s = s.replace("x", "");
                s.parse::<i32>().unwrap_or_default()
            });
            let factory = card_info.get(1).unwrap_or(String::from(""));
            let card_type = card_info.get(4).unwrap_or(String::from(""));
            // let number = card_info.get(5);
            // let number = card_info.get(6);
            
            // let &[number,factory,_,_,card_type,..] = card_info.as_slice();
            println!("number:{:?},enum card type:{:?}",number,format!("{}{}",factory,card_type));
            // println!("card:{:?}",itme);
            itme.clone()
        }).collect();
        // println!("{:?}",servers)
    }
}
