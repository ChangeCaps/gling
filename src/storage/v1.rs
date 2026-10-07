use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::uuid_map::UuidMap;

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
    pub delay: Delay,
    pub reverb: Reverb,
    pub kind: Kind,
    pub random: Random,
    pub path: Option<PathBuf>,
}

impl Default for Sound {
    fn default() -> Self {
        Self {
            name: String::from("new sound"),
            gain: 0.0,
            delay: Delay::default(),
            reverb: Reverb::default(),
            kind: Kind::Trigger,
            random: Random::default(),
            path: Default::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Random {
    pub limit: u32,
    pub rate: f32,
    pub volume: f32,
    pub panning: f32,
}

impl Default for Random {
    fn default() -> Self {
        Self {
            limit: 64,
            rate: 0.05,
            volume: 0.0,
            panning: 0.5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Delay {
    pub time: f32,
    pub feedback: f32,
    pub mix: f32,
}

impl Default for Delay {
    fn default() -> Self {
        Self {
            time: 0.5,
            feedback: -6.0,
            mix: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Reverb {
    pub feedback: f32,
    pub damping: f32,
    pub width: f32,
    pub mix: f32,
}

impl Default for Reverb {
    fn default() -> Self {
        Self {
            feedback: 0.9,
            damping: 0.1,
            width: 1.0,
            mix: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Music,
    Ambient,
    Random,
    Trigger,
}

impl Kind {
    pub fn name(&self) -> &'static str {
        match self {
            Kind::Music => "Music",
            Kind::Ambient => "Ambient",
            Kind::Random => "Random",
            Kind::Trigger => "Trigger",
        }
    }

    pub fn is_looping(&self) -> bool {
        matches!(self, Self::Music | Self::Ambient)
    }

    pub fn is_triggered(&self) -> bool {
        matches!(self, Self::Random | Self::Trigger)
    }
}
