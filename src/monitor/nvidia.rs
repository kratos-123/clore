use std::{
    ops::{Deref, DerefMut},
    process::Command,
    str::FromStr,
};

use crate::server::clore::model::CardType;

pub struct GeForce {
    id: u32,
    uuid: String,
    card_type: CardType,
}

pub struct GeForces(Vec<GeForce>);

impl Deref for GeForces {
    type Target = Vec<GeForce>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GeForces {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl GeForces {
    pub fn new() -> GeForces {
        let output = GeForces::command();
        if output.is_err() {
            panic!("无法识别显卡信息:{:?}",output);
        }
        assert!(output.is_ok());
        //GPU 0: NVIDIA GeForce RTX 4070 Ti SUPER (UUID: GPU-5e4c623f-998d-912c-3743-3465506f63ad)
        let nvidia_info = output.unwrap();
        let nvidias = nvidia_info
            .split("\n")
            .map(|nvidia| nvidia.trim())
            .filter(|nvidia| !nvidia.is_empty())
            .map(|nvidia| {
                let card = nvidia
                    .split(" ")
                    .map(|s| {
                        s.trim()
                            .trim_matches(|c| c == ':' || c == '(' || c == '\\')
                            .to_string()
                    })
                    .collect::<Vec<String>>();
                match &card[..] {
                    [_, id, card, _, _, card_type, _, _, _, uuid, _, ..] => {
                        let card_type = format!("{}{}", card, card_type);
                        GeForce {
                            id: id.parse::<u32>().unwrap_or_default(),
                            uuid: uuid.clone(),
                            card_type: CardType::from_str(&card_type).unwrap(),
                        }
                    }
                    _ => {
                        panic!("识别显卡错误:{:?}", nvidia);
                    }
                }
            })
            .collect::<Vec<GeForce>>();
        GeForces(nvidias)
    }

    fn command() -> Result<String, String> {
        let output = Command::new("nvidia-smi")
            .arg("-L")
            .output()
            .map_err(|e| e.to_string())?
            .stdout;
        let nvidia_info = String::from_utf8(output).map_err(|e| e.to_string())?;

        Ok(nvidia_info)
    }
}
