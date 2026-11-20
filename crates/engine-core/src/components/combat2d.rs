use super::damage::AABB2D;
use super::flame::FlameEnergy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FireAbilityType {
    Fireball,
    FlameBurst,
    FlameWave,
    FlameShield,
}

/// 4-tier upgrade level defined in GDD Section 8.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UpgradeLevel {
    Level1,
    Level2,
    Level3,
    Level4,
}

impl UpgradeLevel {
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Level1 => Some(Self::Level2),
            Self::Level2 => Some(Self::Level3),
            Self::Level3 => Some(Self::Level4),
            Self::Level4 => None,
        }
    }

    pub fn upgrade_cost(&self) -> u32 {
        match self {
            Self::Level1 => 150,  // Cost to go to Lvl 2
            Self::Level2 => 350,  // Cost to go to Lvl 3
            Self::Level3 => 750,  // Cost to go to Lvl 4
            Self::Level4 => 0,
        }
    }

    pub fn power_multiplier(&self) -> f32 {
        match self {
            Self::Level1 => 1.0,
            Self::Level2 => 1.35,
            Self::Level3 => 1.80,
            Self::Level4 => 2.40,
        }
    }
}

/// Individual ability stats with cooldown timer and upgrade tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilitySpec {
    pub ability_type: FireAbilityType,
    pub level: UpgradeLevel,
    pub base_damage: f32,
    pub energy_cost: f32,
    pub cooldown_seconds: f32,
    pub current_cooldown: f32,
}

impl AbilitySpec {
    pub fn new(ability_type: FireAbilityType, base_damage: f32, energy_cost: f32, cooldown: f32) -> Self {
        Self {
            ability_type,
            level: UpgradeLevel::Level1,
            base_damage,
            energy_cost,
            cooldown_seconds: cooldown,
            current_cooldown: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.current_cooldown > 0.0 {
            self.current_cooldown = (self.current_cooldown - dt).max(0.0);
        }
    }

    pub fn is_ready(&self) -> bool {
        self.current_cooldown <= 0.0
    }

    pub fn effective_damage(&self) -> f32 {
        self.base_damage * self.level.power_multiplier()
    }

    /// Attempts to cast ability. Deducts flame energy and sets cooldown timer.
    pub fn trigger(&mut self, flame: &mut FlameEnergy) -> bool {
        if self.is_ready() && flame.consume(self.energy_cost) {
            self.current_cooldown = self.cooldown_seconds;
            true
        } else {
            false
        }
    }

    pub fn upgrade(&mut self, shards: &mut u32) -> bool {
        if let Some(next_lvl) = self.level.next() {
            let cost = self.level.upgrade_cost();
            if *shards >= cost {
                *shards -= cost;
                self.level = next_lvl;
                return true;
            }
        }
        false
    }
}

/// The player's combat loadout carrying all 4 core fire abilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatLoadout {
    pub fireball: AbilitySpec,
    pub flame_burst: AbilitySpec,
    pub flame_wave: AbilitySpec,
    pub flame_shield: AbilitySpec,

    // Shield active state
    pub is_shield_active: bool,
    pub shield_duration_remaining: f32,
}

impl Default for CombatLoadout {
    fn default() -> Self {
        Self {
            // Fireball: Ranged single-target projectile
            fireball: AbilitySpec::new(FireAbilityType::Fireball, 45.0, 12.0, 0.45),
            // Flame Burst: Radial AoE explosion around player
            flame_burst: AbilitySpec::new(FireAbilityType::FlameBurst, 85.0, 28.0, 1.80),
            // Flame Wave: Sweeping horizontal ground wave
            flame_wave: AbilitySpec::new(FireAbilityType::FlameWave, 60.0, 22.0, 1.20),
            // Flame Shield: Barrier absorbing damage for 3 seconds
            flame_shield: AbilitySpec::new(FireAbilityType::FlameShield, 0.0, 30.0, 6.00),
            is_shield_active: false,
            shield_duration_remaining: 0.0,
        }
    }
}

impl CombatLoadout {
    pub fn tick(&mut self, dt: f32) {
        self.fireball.tick(dt);
        self.flame_burst.tick(dt);
        self.flame_wave.tick(dt);
        self.flame_shield.tick(dt);

        if self.is_shield_active {
            self.shield_duration_remaining = (self.shield_duration_remaining - dt).max(0.0);
            if self.shield_duration_remaining == 0.0 {
                self.is_shield_active = false;
            }
        }
    }

    pub fn activate_shield(&mut self, flame: &mut FlameEnergy) -> bool {
        if self.flame_shield.trigger(flame) {
            self.is_shield_active = true;
            self.shield_duration_remaining = 3.0 * self.flame_shield.level.power_multiplier();
            true
        } else {
            false
        }
    }
}

/// Active projectile or fire wave traveling in the game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveFlameAttack {
    pub ability_type: FireAbilityType,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub damage: f32,
    pub lifetime: f32,
    pub penetrates: bool,
}

impl ActiveFlameAttack {
    pub fn new_fireball(x: f32, y: f32, direction_x: f32, damage: f32) -> Self {
        Self {
            ability_type: FireAbilityType::Fireball,
            x,
            y,
            vx: direction_x * 480.0,
            vy: 0.0,
            radius: 12.0,
            damage,
            lifetime: 1.5,
            penetrates: false,
        }
    }

    pub fn new_flame_wave(x: f32, y: f32, direction_x: f32, damage: f32) -> Self {
        Self {
            ability_type: FireAbilityType::FlameWave,
            x,
            y,
            vx: direction_x * 320.0,
            vy: 0.0,
            radius: 28.0,
            damage,
            lifetime: 2.2,
            penetrates: true,
        }
    }

    pub fn bounding_box(&self) -> AABB2D {
        AABB2D::from_center_size(self.x, self.y, self.radius * 2.0, self.radius * 2.0)
    }

    pub fn step(&mut self, dt: f32) -> bool {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.lifetime -= dt;
        self.lifetime > 0.0
    }
}
