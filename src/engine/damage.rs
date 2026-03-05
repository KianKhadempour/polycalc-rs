use crate::{data::unit_definition::UnitDefinition, state::unit_instance::UnitInstance};

pub fn calculate_damage(
    attacker: &UnitInstance,
    defender: &UnitInstance,
    attacker_unit: &UnitDefinition,
    defender_unit: &UnitDefinition,
) -> CombatResult {
    let attacker_force =
        attacker_unit.stats.attack * attacker.hp * 1000 / attacker_unit.stats.max_hp; // 1milx
    let defender_force = defender_unit.stats.defense  // 1000x
        * defender.hp                                      // 1milx
        * defender.statuses.defense_bonus()                // 1bilx
        / defender_unit.stats.max_hp; // 1milx

    let total_damage = attacker_force + defender_force; // 1milx

    // rounded
    let damage_to_attacker = (defender_force * defender_unit.stats.defense * 45 / 10000
        + (total_damage / 2))
        / total_damage
        * 1000;
    let damage_to_defender = (attacker_force * attacker_unit.stats.attack * 45 / 10000
        + (total_damage / 2))
        / total_damage
        * 1000;

    CombatResult {
        damage_to_attacker,
        damage_to_defender,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CombatResult {
    pub damage_to_attacker: i64,
    pub damage_to_defender: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::loader::{Loader, UnitRegistry},
        state::status_effect::StatusFlags,
    };
    use std::sync::OnceLock;

    static REGISTRY: OnceLock<UnitRegistry> = OnceLock::new();

    fn registry() -> &'static UnitRegistry {
        REGISTRY.get_or_init(|| {
            let loader = Loader::new("unit_data.toml").unwrap();
            loader.load()
        })
    }

    fn cr(a: i64, d: i64) -> CombatResult {
        CombatResult {
            damage_to_attacker: a * 1000,
            damage_to_defender: d * 1000,
        }
    }

    fn u(unit: &'static str, hp: i64) -> UnitInstance {
        let registry = registry();
        let unit_id = registry.name_to_id[unit];
        UnitInstance::new(unit_id, hp, StatusFlags::empty())
    }

    fn u_buffed(unit: &'static str, hp: i64, flags: StatusFlags) -> UnitInstance {
        let registry = registry();
        let unit_id = registry.name_to_id[unit];
        UnitInstance::new(unit_id, hp, flags)
    }

    macro_rules! test_combat {
        ($test_name:ident, $att_name:expr, $att_hp:expr, $def_name:expr, $def_hp:expr, $exp_att_dmg:expr, $exp_def_dmg:expr) => {
            #[test]
            fn $test_name() {
                let registry = registry();
                let attacker = u($att_name, $att_hp);
                let defender = u($def_name, $def_hp);
                let attacker_unit = &registry.definitions[attacker.unit_id.0];
                let defender_unit = &registry.definitions[defender.unit_id.0];
                assert_eq!(
                    calculate_damage(&attacker, &defender, attacker_unit, defender_unit),
                    cr($exp_att_dmg, $exp_def_dmg),
                    "Failed on {} ({} HP) attacking {} ({} HP)",
                    $att_name,
                    $att_hp,
                    $def_name,
                    $def_hp
                );
            }
        };
    }
    test_combat!(ar_vs_ar_full, "Archer", 10, "Archer", 10, 2, 6);
    test_combat!(wa_vs_wa_full, "Warrior", 10, "Warrior", 10, 5, 5);
    test_combat!(tr_vs_tr_full, "Tridention", 10, "Tridention", 10, 1, 8);

    test_combat!(wa_inj5_vs_wa_full, "Warrior", 5, "Warrior", 10, 6, 3);
    test_combat!(ar_inj5_vs_ar_full, "Archer", 5, "Archer", 10, 2, 5);

    test_combat!(wa_full_vs_wa_inj5, "Warrior", 10, "Warrior", 5, 3, 6);
    test_combat!(ar_full_vs_ar_inj5, "Archer", 10, "Archer", 5, 1, 7);

    test_combat!(wa_inj5_vs_wa_inj5, "Warrior", 5, "Warrior", 5, 5, 5);

    test_combat!(wa_1hp_vs_wa_full, "Warrior", 1, "Warrior", 10, 8, 1);
    test_combat!(wa_full_vs_wa_1hp, "Warrior", 10, "Warrior", 1, 1, 8);
    test_combat!(tr_1hp_vs_tr_full, "Tridention", 1, "Tridention", 10, 4, 2);

    test_combat!(wa_vs_ar, "Warrior", 10, "Archer", 10, 2, 6);
    test_combat!(ar_vs_wa, "Archer", 10, "Warrior", 10, 5, 5);

    #[test]
    fn warrior_vs_warrior_with_defense_bonus() {
        let registry = registry();

        let attacker = u("Warrior", 10);
        let defender = u_buffed("Warrior", 10, StatusFlags::FORTIFIED);
        let attacker_unit = &registry.definitions[attacker.unit_id.0];
        let defender_unit = &registry.definitions[defender.unit_id.0];

        let result = calculate_damage(&attacker, &defender, attacker_unit, defender_unit);

        assert_eq!(result, cr(5, 4));
    }
}
