use crate::{data::loader::UnitId, state::status_effect::StatusFlags};

#[derive(Debug)]
pub struct UnitInstance {
    pub unit_id: UnitId,
    pub hp: i64,
    pub statuses: StatusFlags,
}

impl UnitInstance {
    pub fn new(unit_id: UnitId, hp: i64, statuses: StatusFlags) -> Self {
        Self {
            unit_id,
            hp,
            statuses,
        }
    }

    pub fn new_no_status(unit_id: UnitId, hp: i64) -> Self {
        Self {
            unit_id,
            hp,
            statuses: StatusFlags::empty(),
        }
    }

    pub fn new_10_hp(unit_id: UnitId) -> Self {
        Self {
            unit_id,
            hp: 10000,
            statuses: StatusFlags::empty(),
        }
    }
}
