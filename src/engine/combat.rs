use crate::{
    data::{trait_id::TraitId, unit_definition::UnitDefinition},
    engine::damage::calculate_damage,
    state::{status_effect::StatusFlags, unit_instance::UnitInstance},
};

fn before_attack(
    attacker: &mut UnitInstance,
    defender: &UnitInstance,
    attacker_def: &UnitDefinition,
    defender_def: &UnitDefinition,
) -> i64 {
    let mut pre_damage = 0i64;
    for &t in &defender_def.traits {
        match t {
            TraitId::PreemptiveRetaliation => {
                let result = calculate_damage(defender, attacker, defender_def, attacker_def);
                attacker.hp -= result.damage_to_defender;
                pre_damage += result.damage_to_defender;
            }
            _ => {}
        }
    }
    pre_damage
}

fn after_attack(attacker_def: &UnitDefinition, defender: &mut UnitInstance) {
    for &t in &attacker_def.traits {
        match t {
            TraitId::PoisonOnHit => {
                defender.statuses |= StatusFlags::POISONED;
            }
            TraitId::FreezeOnHit => {
                defender.statuses |= StatusFlags::FROZEN;
            }
            _ => {}
        }
    }
}

fn retaliation_suppressed(attacker_def: &UnitDefinition, defender_def: &UnitDefinition) -> bool {
    attacker_def.traits.contains(&TraitId::NoRetaliationTaken)
        || !defender_def.traits.contains(&TraitId::Retaliates)
}

pub fn resolve_attack(
    attacker: &mut UnitInstance,
    defender: &mut UnitInstance,
    attacker_def: &UnitDefinition,
    defender_def: &UnitDefinition,
) {
    // Step 1: before_attack — defender's traits may act preemptively
    before_attack(attacker, defender, attacker_def, defender_def);
    if attacker.hp <= 0 {
        return;
    }

    // Step 2: Primary attack
    let result = calculate_damage(attacker, defender, attacker_def, defender_def);
    defender.hp -= result.damage_to_defender;

    // Step 3: Defender death check
    if defender.hp <= 0 {
        return;
    }

    // Step 4: after_attack — apply on-hit effects to defender
    after_attack(attacker_def, defender);

    // Step 5: Retaliation
    if !retaliation_suppressed(attacker_def, defender_def) {
        attacker.hp -= result.damage_to_attacker;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::{
            loader::{Loader, UnitId, UnitRegistry},
            stats::Stats,
            unit_definition::UnitDefinition,
        },
        state::status_effect::StatusFlags,
    };
    use smallvec;
    use std::sync::OnceLock;

    static REGISTRY: OnceLock<UnitRegistry> = OnceLock::new();

    fn registry() -> &'static UnitRegistry {
        REGISTRY.get_or_init(|| {
            let loader = Loader::new("unit_data.toml").unwrap();
            loader.load()
        })
    }

    fn u(unit: &'static str, hp: i64) -> UnitInstance {
        let registry = registry();
        let unit_id = registry.name_to_id[unit];
        UnitInstance::new(unit_id, hp, StatusFlags::empty())
    }

    fn make_def(id_idx: usize, attack: i64, defense: i64, hp: i64, traits: &[TraitId]) -> UnitDefinition {
        UnitDefinition {
            id: UnitId(id_idx),
            stats: Stats {
                attack,
                defense,
                max_hp: hp,
                range: 1,
                cost: 1,
            },
            traits: smallvec::SmallVec::from_slice(traits),
        }
    }

    fn make_unit(id_idx: usize, hp: i64) -> UnitInstance {
        UnitInstance::new(UnitId(id_idx), hp, StatusFlags::empty())
    }

    // Basic attack, no traits: Catapult (no traits) attacks Catapult — only defender takes damage
    #[test]
    fn basic_attack_no_retaliation() {
        let registry = registry();
        let mut attacker = u("Catapult", 10000);
        let mut defender = u("Catapult", 10000);
        let attacker_def = &registry.definitions[attacker.unit_id.0];
        let defender_def = &registry.definitions[defender.unit_id.0];

        let attacker_hp_before = attacker.hp;
        resolve_attack(&mut attacker, &mut defender, attacker_def, defender_def);

        assert!(defender.hp < 10000, "Defender should take damage");
        assert_eq!(attacker.hp, attacker_hp_before, "Attacker should take no retaliation from Catapult (no Retaliates)");
    }

    // No retaliation: Catapult (no Retaliates) vs Archer — attacker takes no retaliation
    #[test]
    fn no_retaliation_when_defender_lacks_trait() {
        let registry = registry();
        let mut attacker = u("Catapult", 10);
        let mut defender = u("Catapult", 10);
        let attacker_def = &registry.definitions[attacker.unit_id.0];
        let defender_def = &registry.definitions[defender.unit_id.0];

        let attacker_hp_before = attacker.hp;
        resolve_attack(&mut attacker, &mut defender, attacker_def, defender_def);

        assert!(defender.hp < 10, "Defender should take damage");
        assert_eq!(attacker.hp, attacker_hp_before, "Attacker should take no retaliation");
    }

    // Retaliation: Warrior vs Warrior (both have Retaliates)
    #[test]
    fn retaliation_both_warriors() {
        let registry = registry();
        let mut attacker = u("Warrior", 10000);
        let mut defender = u("Warrior", 10000);
        let attacker_def = &registry.definitions[attacker.unit_id.0];
        let defender_def = &registry.definitions[defender.unit_id.0];

        let attacker_hp_before = attacker.hp;
        resolve_attack(&mut attacker, &mut defender, attacker_def, defender_def);

        assert!(defender.hp < 10000, "Defender should take damage");
        assert!(attacker.hp < attacker_hp_before, "Attacker should take retaliation");
    }

    // NoRetaliationTaken suppresses retaliation
    #[test]
    fn no_retaliation_taken_suppresses_retaliation() {
        let registry = registry();
        let mut attacker = u("Dagger", 10000);
        let mut defender = u("Warrior", 10000);
        let attacker_def = &registry.definitions[attacker.unit_id.0];
        let defender_def = &registry.definitions[defender.unit_id.0];

        let attacker_hp_before = attacker.hp;
        resolve_attack(&mut attacker, &mut defender, attacker_def, defender_def);

        assert!(defender.hp < 10000, "Defender should take damage");
        assert_eq!(attacker.hp, attacker_hp_before, "Attacker with NoRetaliationTaken takes no retaliation");
    }

    // PreemptiveRetaliation: Jelly attacks — defender strikes first
    #[test]
    fn preemptive_retaliation_reduces_attacker_hp_first() {
        let registry = registry();
        // Warrior attacks Jelly (has PreemptiveRetaliation)
        let mut attacker = u("Warrior", 10);
        let mut defender = u("Jelly", 10);
        let attacker_def = &registry.definitions[attacker.unit_id.0];
        let defender_def = &registry.definitions[defender.unit_id.0];

        // Calculate what damage Jelly would do preemptively
        let preemptive_result = calculate_damage(&defender, &attacker, defender_def, attacker_def);
        let expected_attacker_hp_after_preemptive = attacker.hp - preemptive_result.damage_to_defender;

        resolve_attack(&mut attacker, &mut defender, attacker_def, defender_def);

        // Attacker HP should be reduced by at least the preemptive damage
        assert!(
            attacker.hp <= expected_attacker_hp_after_preemptive,
            "Attacker HP should be reduced by preemptive retaliation"
        );
    }

    // PoisonOnHit: attacker with PoisonOnHit poisons the defender
    #[test]
    fn poison_on_hit_sets_poisoned_status() {
        let registry = registry();
        // Exida has PoisonOnHit
        let mut attacker = u("Exida", 10000);
        let mut defender = u("Warrior", 10000);
        let attacker_def = &registry.definitions[attacker.unit_id.0];
        let defender_def = &registry.definitions[defender.unit_id.0];

        resolve_attack(&mut attacker, &mut defender, attacker_def, defender_def);

        assert!(
            defender.statuses.contains(StatusFlags::POISONED),
            "Defender should be poisoned after being hit by Exida"
        );
    }

    // FreezeOnHit: attacker with FreezeOnHit freezes the defender
    #[test]
    fn freeze_on_hit_sets_frozen_status() {
        let registry = registry();
        // IceArcher has FreezeOnHit
        let mut attacker = u("IceArcher", 10);
        let mut defender = u("Warrior", 10);
        let attacker_def = &registry.definitions[attacker.unit_id.0];
        let defender_def = &registry.definitions[defender.unit_id.0];

        resolve_attack(&mut attacker, &mut defender, attacker_def, defender_def);

        assert!(
            defender.statuses.contains(StatusFlags::FROZEN),
            "Defender should be frozen after being hit by IceArcher"
        );
    }

    // Defender killed — no retaliation
    #[test]
    fn defender_killed_no_retaliation() {
        // Use a manually constructed unit with 1 HP defender and powerful attacker
        let attacker_def = make_def(0, 5000, 2000, 10000, &[]);
        let defender_def = make_def(1, 2000, 2000, 10000, &[TraitId::Retaliates]);

        let mut attacker = make_unit(0, 10000);
        let mut defender = make_unit(1, 1); // 1 HP — will die in one hit

        let attacker_hp_before = attacker.hp;
        resolve_attack(&mut attacker, &mut defender, &attacker_def, &defender_def);

        assert!(defender.hp <= 0, "Defender should be dead");
        assert_eq!(attacker.hp, attacker_hp_before, "Attacker should take no retaliation after killing defender");
    }

    // Defender killed — no on-hit effects applied to dead defender
    #[test]
    fn defender_killed_no_on_hit_effects() {
        let attacker_def = make_def(0, 5000, 2000, 10000, &[TraitId::PoisonOnHit, TraitId::FreezeOnHit]);
        let defender_def = make_def(1, 2000, 2000, 10000, &[]);

        let mut attacker = make_unit(0, 10000);
        let mut defender = make_unit(1, 1); // 1 HP — will die

        resolve_attack(&mut attacker, &mut defender, &attacker_def, &defender_def);

        assert!(defender.hp <= 0, "Defender should be dead");
        assert!(
            !defender.statuses.contains(StatusFlags::POISONED),
            "Dead defender should not be poisoned"
        );
        assert!(
            !defender.statuses.contains(StatusFlags::FROZEN),
            "Dead defender should not be frozen"
        );
    }

    // Attacker killed by preemptive retaliation — defender takes no damage
    #[test]
    fn attacker_killed_by_preemptive_no_primary_damage() {
        let attacker_def = make_def(0, 2000, 2000, 10000, &[]);
        let defender_def = make_def(1, 5000, 3000, 10000, &[TraitId::PreemptiveRetaliation]);

        let mut attacker = make_unit(0, 1); // 1 HP — will die to preemptive strike
        let mut defender = make_unit(1, 10000);

        let defender_hp_before = defender.hp;
        resolve_attack(&mut attacker, &mut defender, &attacker_def, &defender_def);

        assert!(attacker.hp <= 0, "Attacker should be dead from preemptive retaliation");
        assert_eq!(
            defender.hp, defender_hp_before,
            "Defender should take no damage since attacker died from preemptive retaliation"
        );
    }

    // PreemptiveRetaliation does NOT consume normal Retaliates
    #[test]
    fn preemptive_retaliation_and_retaliates_both_apply() {
        let attacker_def = make_def(0, 2000, 2000, 10000, &[]);
        let defender_def = make_def(
            1,
            2000,
            2000,
            10000,
            &[TraitId::PreemptiveRetaliation, TraitId::Retaliates],
        );

        let mut attacker = make_unit(0, 10000);
        let mut defender = make_unit(1, 10000);

        let attacker_hp_before = attacker.hp;
        resolve_attack(&mut attacker, &mut defender, &attacker_def, &defender_def);

        // Attacker should have taken damage from both preemptive AND regular retaliation
        // so HP should be less than after just one of those
        let preemptive_result = calculate_damage(
            &make_unit(1, 10000),
            &make_unit(0, 10000),
            &defender_def,
            &attacker_def,
        );
        let preemptive_damage = preemptive_result.damage_to_defender;

        let primary_result = calculate_damage(
            &make_unit(0, 10000 - preemptive_damage),
            &make_unit(1, 10000),
            &attacker_def,
            &defender_def,
        );
        let retaliation_damage = primary_result.damage_to_attacker;

        let expected_attacker_hp = attacker_hp_before - preemptive_damage - retaliation_damage;
        assert_eq!(
            attacker.hp, expected_attacker_hp,
            "Attacker should take both preemptive and regular retaliation damage"
        );
    }
}
