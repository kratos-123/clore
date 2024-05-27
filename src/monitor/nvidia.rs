use std::{
    ops::{Deref, DerefMut},
    process::Command,
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::server::clore::model::CardType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeForce {
    CARD {
        id: u32,
        uuid: String,
        card_type: CardType,
    },
    ERROR(String),
}

#[derive(Debug, Serialize, Deserialize)]
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
        let nvidias = match output {
            Ok(nvidia_info) => nvidia_info
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
                    info!("{:?}", card);
                    match &card[..] {
                        [_, id, card, _, _, card_type, _flag1, _flag2, _, uuid, ..] => {
                            let card_type = format!("{}{}S", card, card_type);
                            let geforce = GeForce::CARD {
                                id: id.parse::<u32>().unwrap_or_default(),
                                uuid: uuid.clone(),
                                card_type: CardType::from_str(&card_type).unwrap(),
                            };
                            geforce
                        }
                        [_, id, card, _, _, card_type, _flag1, _, uuid, ..] => {
                            let card_type = format!("{}{}TI", card, card_type);
                            let geforce = GeForce::CARD {
                                id: id.parse::<u32>().unwrap_or_default(),
                                uuid: uuid.clone(),
                                card_type: CardType::from_str(&card_type).unwrap(),
                            };
                            geforce
                        }
                        [_, id, card, _, _, card_type, _, uuid, ..] => {
                            let card_type = format!("{}{}", card, card_type);
                            let geforce = GeForce::CARD {
                                id: id.parse::<u32>().unwrap_or_default(),
                                uuid: uuid.clone(),
                                card_type: CardType::from_str(&card_type).unwrap(),
                            };
                            geforce
                        }
                        _ => {
                            let e = format!("识别显卡错误:{:?}", card);
                            error!(e);
                            GeForce::ERROR(e)
                        }
                    }
                })
                .collect::<Vec<GeForce>>(),
            Err(e) => vec![GeForce::ERROR(e)],
        };
        GeForces(nvidias)
    }

    fn command() -> Result<String, String> {
        let output = r"
GPU 0: NVIDIA GeForce RTX 4070 (UUID: GPU-5e4c623f-998d-912c-3743-3465506f63ad)
GPU 1: NVIDIA GeForce RTX 4070 Ti (UUID: GPU-5e4c623f-998d-912c-3743-3465506f63ad)
GPU 2: NVIDIA GeForce RTX 4070 Ti SUPER (UUID: GPU-5e4c623f-998d-912c-3743-3465506f63ad)
        ";
        return Ok(output.to_string());
        let output = Command::new("nvidia-smi")
            .arg("-L")
            .output()
            .map_err(|e| e.to_string())?
            .stdout;
        let nvidia_info = String::from_utf8(output).map_err(|e| e.to_string())?;

        Ok(nvidia_info)
    }
}
