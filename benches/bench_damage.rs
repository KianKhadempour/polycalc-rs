use std::hint::black_box;

use combat_core::{
    data::loader::{Loader, UnitRegistry},
    engine::damage::calculate_damage,
    state::{status_effect::StatusFlags, unit_instance::UnitInstance},
};
use criterion::{Criterion, criterion_group, criterion_main};

pub fn criterion_benchmark(c: &mut Criterion) {
    let loader = Loader::new("unit_data.toml").unwrap();
    let registry = loader.load();

    macro_rules! bench_damage_s {
        ($bench_name:expr, $att_name:expr, $att_hp:expr, $att_s:expr, $def_name:expr, $def_hp:expr, $def_s:expr) => {
            let attacker = u_buffed($att_name, $att_hp, $att_s, &registry);
            let defender = u_buffed($def_name, $def_hp, $def_s, &registry);
            let attacker_unit = &registry.definitions[attacker.unit_id.0];
            let defender_unit = &registry.definitions[defender.unit_id.0];

            let att_hp = attacker.hp;
            let att_max_hp = attacker_unit.stats.max_hp;
            let def_hp = defender.hp;
            let def_max_hp = defender_unit.stats.max_hp;
            let attack = attacker_unit.stats.attack;
            let defense = defender_unit.stats.defense;
            let defense_bonus = defender.statuses.defense_bonus();

            c.bench_function($bench_name, |b| {
                b.iter(|| {
                    calculate_damage(
                        black_box(att_hp),
                        black_box(att_max_hp),
                        black_box(def_hp),
                        black_box(def_max_hp),
                        black_box(attack),
                        black_box(defense),
                        black_box(defense_bonus),
                    )
                });
            });
        };
    }

    macro_rules! bench_damage {
        ($bench_name:expr, $att_name:expr, $def_name:expr) => {
            bench_damage_s!(
                $bench_name,
                $att_name,
                10,
                StatusFlags::empty(),
                $def_name,
                10,
                StatusFlags::empty()
            );
        };
    }

    bench_damage!("wa wa", "Warrior", "Warrior");
    bench_damage!("wa ar", "Warrior", "Archer");
    bench_damage_s!(
        "wa wa d",
        "Warrior",
        10,
        StatusFlags::empty(),
        "Warrior",
        10,
        StatusFlags::FORTIFIED
    );
    bench_damage_s!(
        "wa wa 5 d",
        "Warrior",
        10,
        StatusFlags::empty(),
        "Warrior",
        5,
        StatusFlags::FORTIFIED
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn u_buffed(
    unit: &'static str,
    hp: i64,
    flags: StatusFlags,
    registry: &UnitRegistry,
) -> UnitInstance {
    let unit_id = registry.name_to_id[unit];
    UnitInstance::new(unit_id, hp, flags)
}
