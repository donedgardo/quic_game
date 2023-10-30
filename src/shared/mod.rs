use serde::{Deserialize, Serialize};
use bevy::prelude::Resource;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SharedMessage {
    TestMessage(String),
}

#[derive(Resource, Debug, Clone, Default)]
pub struct Port(pub u16);
