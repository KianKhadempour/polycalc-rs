use crate::state::unit_instance::UnitInstance;

pub struct BattleState {
    pub attackers: Vec<UnitInstance>,
    pub defenders: Vec<UnitInstance>,
}
