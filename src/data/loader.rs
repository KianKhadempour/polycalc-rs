use std::{
    collections::HashMap,
    fs::{exists, read},
};

use indexmap::IndexMap;
use serde::Deserialize;

use smallvec::SmallVec;
use thiserror::Error;
use toml;

use crate::data::{stats::Stats, trait_id::TraitId, unit_definition::UnitDefinition};

pub struct Loader {
    data_path: &'static str,
}

impl Loader {
    pub fn new(data_path: &'static str) -> Result<Self, FileNotFoundError> {
        if !exists(data_path).expect("something went very wrong trying to check if the file exists")
        {
            return Err(FileNotFoundError {
                file_path: data_path.to_owned(),
            });
        }

        Ok(Self { data_path })
    }
    pub fn load(&self) -> UnitRegistry {
        let data_file = read(self.data_path).expect("unable to read provided data file");

        let data: IndexMap<String, UnitData> =
            toml::from_slice(&data_file).expect("unable to deserialize unit data");

        let mut definitions: Vec<UnitDefinition> = Vec::with_capacity(data.len());
        let mut name_to_id: HashMap<String, UnitId> = HashMap::new();

        for (i, (k, v)) in data.into_iter().enumerate() {
            let stats = Stats {
                attack: v.attack,
                defense: v.defense,
                max_hp: v.hp,
                range: v.range,
                cost: v.cost,
            };

            let unit_id = UnitId(i);

            name_to_id.insert(k, unit_id);
            definitions.push(UnitDefinition {
                id: unit_id,
                stats,
                traits: SmallVec::from_vec(v.traits),
            });
        }

        UnitRegistry {
            definitions,
            name_to_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitId(pub usize);

#[derive(Debug)]
pub struct UnitRegistry {
    pub definitions: Vec<UnitDefinition>,
    pub name_to_id: HashMap<String, UnitId>,
}

#[derive(Deserialize, Debug)]
struct UnitData {
    attack: i64,
    defense: i64,
    hp: i64,
    range: u8,
    cost: u8,
    traits: Vec<TraitId>,
}

#[derive(Error, Debug)]
#[error("unable to find unit data file at path \"{file_path}\"")]
pub struct FileNotFoundError {
    pub file_path: String,
}
