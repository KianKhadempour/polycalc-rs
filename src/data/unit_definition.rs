use crate::data::loader::UnitId;
use crate::data::stats::Stats;
use crate::data::trait_id::TraitId;

use smallvec::SmallVec;

#[derive(Debug)]
pub struct UnitDefinition {
    pub id: UnitId,
    pub stats: Stats,
    pub traits: SmallVec<[TraitId; 4]>,
}
