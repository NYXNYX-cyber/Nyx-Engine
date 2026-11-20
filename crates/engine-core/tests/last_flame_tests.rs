use engine_core::components::*;
use engine_core::generators::*;

#[test]
fn test_flame_energy_and_firefly_rescue() {
    let mut flame = FlameEnergy::new(100.0, 5.0, 150.0);
    assert_eq!(flame.state(), FlameState::Radiant);

    // Consume down to stable
    assert!(flame.consume(40.0));
    assert_eq!(flame.state(), FlameState::Stable);

    // Consume down to flickering
    assert!(flame.consume(35.0));
    assert_eq!(flame.state(), FlameState::Flickering);

    // Consume down to critical
    assert!(flame.consume(20.0));
    assert_eq!(flame.state(), FlameState::Critical);

    // Consume to zero -> Extinguished
    assert!(flame.consume(5.0));
    assert_eq!(flame.state(), FlameState::Extinguished);

    // Test Firefly Rescue (1x Lifeline from GDD 4.3)
    let mut rescue = FireflyRescue::new(0.40, 2.5);
    assert!(rescue.has_lifeline);
    assert!(rescue.trigger_rescue(&mut flame));

    // Flame should now be rekindled to 40% (Stable)
    assert_eq!(flame.current, 40.0);
    assert_eq!(flame.state(), FlameState::Stable);
    assert!(!rescue.has_lifeline); // Lifeline spent!

    // Second extinction: rescue MUST fail (Permanent death)
    flame.consume(40.0);
    assert_eq!(flame.state(), FlameState::Extinguished);
    assert!(!rescue.trigger_rescue(&mut flame));
}

#[test]
fn test_movement_double_jump_and_coyote() {
    let mut movement = PlatformerMovement2D::new(100.0, 20.0);
    movement.on_landed(20.0);
    assert!(movement.is_grounded);
    assert_eq!(movement.jumps_remaining, 2);

    // First jump
    movement.press_jump();
    movement.step_physics(0.016);
    assert!(!movement.is_grounded);
    assert!(movement.vy > 0.0);
    assert_eq!(movement.jumps_remaining, 1);

    // Double Jump in mid-air (GDD 4.1)
    movement.press_jump();
    movement.step_physics(0.016);
    assert!(movement.vy > 0.0);
    assert_eq!(movement.jumps_remaining, 0);

    // Third jump attempt mid-air must be ignored
    movement.press_jump();
    movement.step_physics(0.016);
    assert_eq!(movement.jumps_remaining, 0);
}

#[test]
fn test_damage_categories_and_flame_drain() {
    let mut health = Health::new(100.0, 0.0); // 0 invuln for unit test
    let mut flame = FlameEnergy::new(100.0, 0.0, 150.0);

    // 1. Health Damage only
    health.take_damage(20.0, DamageType::HealthDamage, Some(&mut flame));
    assert_eq!(health.current, 80.0);
    assert_eq!(flame.current, 100.0); // Flame untouched

    // 2. Flame Damage only
    health.take_damage(30.0, DamageType::FlameDamage, Some(&mut flame));
    assert_eq!(health.current, 80.0); // HP untouched
    assert_eq!(flame.current, 70.0); // Flame drained!

    // 3. Dual Damage (both HP and Flame affected)
    health.take_damage(40.0, DamageType::DualDamage, Some(&mut flame));
    assert_eq!(health.current, 50.0); // 80 - (40 * 0.75) = 50
    assert_eq!(flame.current, 40.0); // 70 - (40 * 0.75) = 40
}

#[test]
fn test_combat_abilities_and_upgrade_tiers() {
    let mut loadout = CombatLoadout::default();
    let mut flame = FlameEnergy::new(100.0, 0.0, 150.0);
    let mut shards = 1500u32;

    // Trigger Fireball
    assert!(loadout.fireball.trigger(&mut flame));
    assert_eq!(flame.current, 88.0); // 100 - 12
    assert!(!loadout.fireball.is_ready()); // On cooldown

    // Test Upgrade to Level 2
    assert!(loadout.fireball.upgrade(&mut shards));
    assert_eq!(loadout.fireball.level, UpgradeLevel::Level2);
    assert_eq!(shards, 1350); // 1500 - 150

    // Test Upgrade to Level 3
    assert!(loadout.fireball.upgrade(&mut shards));
    assert_eq!(loadout.fireball.level, UpgradeLevel::Level3);
    assert_eq!(shards, 1000); // 1350 - 350

    // Damage increased with upgrade level multiplier
    assert!(loadout.fireball.effective_damage() > loadout.fireball.base_damage);
}

#[test]
fn test_procedural_generators() {
    // 1. Level generator for Chapter 1
    let gen = PlatformerLevelGenerator::new(Chapter::SilentForest, 1337);
    let map = gen.generate(100, 20, 32.0);
    assert_eq!(map.width, 100);
    assert_eq!(map.height, 20);

    // 2. Enemy encounter generator
    let enemies = EncounterGenerator::populate_enemies(&map, Chapter::SilentForest, 1337);
    assert!(!enemies.is_empty());

    // 3. Light and pickup generator
    let (checkpoints, braziers, pickups) = LightAndPickupGenerator::populate(&map, 1337);
    assert!(!checkpoints.is_empty());
    assert!(!braziers.is_empty());
    assert!(!pickups.is_empty());

    // Checkpoint activation test
    let mut cp = checkpoints[0].clone();
    let mut flame = FlameEnergy::new(100.0, 5.0, 150.0);
    flame.consume(70.0);
    cp.activate(&mut flame);
    assert!(cp.is_ignited);
    if cp.checkpoint_type == CheckpointType::Recharge || cp.checkpoint_type == CheckpointType::Dual {
        assert_eq!(flame.current, 100.0);
    }

    // 4. Puzzle obstacle test
    let mut bramble = FlamePuzzleObstacle::new_bramble(1, 200.0, 50.0, 50.0);
    assert!(!bramble.is_cleared);
    assert!(bramble.apply_fire_hit(60.0));
    assert!(bramble.is_cleared);
}
