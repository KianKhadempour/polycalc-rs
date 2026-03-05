use std::str::FromStr;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum TraitId {
    Retaliates,
    ConvertOnHit,
    PoisonOnHit,
    FreezeOnHit,
    NoRetaliationTaken,
    PreemptiveRetaliation,
}

impl FromStr for TraitId {
    type Err = ParseTraitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Retaliates" => Ok(Self::Retaliates),
            "ConvertOnHit" => Ok(Self::ConvertOnHit),
            "PoisonOnHit" => Ok(Self::PoisonOnHit),
            "FreezeOnHit" => Ok(Self::FreezeOnHit),
            "NoRetaliationTaken" => Ok(Self::NoRetaliationTaken),
            "PreemptiveRetaliation" => Ok(Self::PreemptiveRetaliation),
            _ => Err(ParseTraitError { value: s.to_owned() }),
        }
    }
}

#[derive(Error, Debug)]
#[error("unable to parse string \"{value}\" into a TraitId")]
pub struct ParseTraitError {
    value: String,
}
