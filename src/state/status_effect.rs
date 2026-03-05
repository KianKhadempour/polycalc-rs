use bitflags::{bitflags, bitflags_match};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StatusFlags: u32 {
        const POISONED  = 1 << 0;
        const FROZEN    = 1 << 1;
        const VETERAN   = 1 << 2;
        const FORTIFIED = 1 << 3;
        const WALLED    = 1 << 4;
    }
}

impl StatusFlags {
    pub fn defense_bonus(&self) -> i64 {
        if self.contains(Self::POISONED) {
            bitflags_match!(self, {
                &Self::FORTIFIED => 700,
                &Self::WALLED => 2000,
                _ => 500,
            })
        } else {
            bitflags_match!(self, {
                &Self::FORTIFIED => 1500,
                &Self::WALLED => 4000,
                _ => 1000,
            })
        }
    }
}
