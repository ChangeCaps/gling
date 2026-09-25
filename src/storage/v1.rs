use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{scene::Kind, uuid_map::UuidMap};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Data {
    pub scenes: UuidMap<Scene>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Scene {
    pub name: String,
    pub master: Master,
    pub sounds: UuidMap<Sound>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            name: String::from("New scene"),
            master: Default::default(),
            sounds: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Master {
    pub gain: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Sound {
    pub name: String,
    pub gain: f32,
    pub kind: Kind,
    pub rate: f32,
    pub path: Option<PathBuf>,
}

impl Default for Sound {
    fn default() -> Self {
        Self {
            name: String::from("new sound"),
            gain: 0.0,
            kind: Kind::Trigger,
            rate: 0.1,
            path: Default::default(),
        }
    }
}
