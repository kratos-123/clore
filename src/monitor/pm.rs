use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::info;

#[allow(dead_code)]
pub struct Pm2 {
    id: u32,
    name: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Envs {
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pros {
    pm_id: u32,
    name: String,
    pm2_env: Envs,
}

pub enum Action {
    START,
    RESTART,
    SKIP,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Process(Vec<Pros>);

impl Deref for Process {
    type Target = Vec<Pros>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Process {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Process {
    pub fn new() -> Result<Process, String> {
        let output = Command::new("pm2")
            .args(&["jlist"])
            .output()
            .map_err(|e| e.to_string())?;
        let row = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        info!("运行:pm2 jlist,{}", row);
        serde_json::from_str::<Process>(&row).map_err(|e| e.to_string())
    }

    pub fn get_action(&self, name: &str) -> Action {
        let mut action = Action::START;
        for pm2 in self.to_pm2().iter() {
            if pm2.name == name {
                if pm2.status == "online" {
                    action = Action::SKIP;
                } else {
                    action = Action::RESTART;
                }
            }
        }
        action
    }

    pub fn to_pm2(&self) -> Vec<Pm2> {
        let mut pm2 = Vec::<Pm2>::new();
        for pros in (*self).iter() {
            pm2.push(Pm2 {
                id: pros.pm_id,
                name: pros.name.clone(),
                status: pros.pm2_env.status.clone(),
            });
        }
        pm2
    }
}
