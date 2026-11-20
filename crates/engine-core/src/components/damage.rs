use super::flame::FlameEnergy;
use serde::{Deserialize, Serialize};

/// 3 distinct damage categories defined in GDD Section 6.2:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageType {
    /// Pure HP reduction.
    HealthDamage,
    /// Drains player's vital Flame Energy without direct HP loss.
    FlameDamage,
    /// Devastating attacks that simultaneously drain HP and Flame Energy.
    DualDamage,
}

/// Standard Health component for player and enemies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub invulnerability_timer: f32,
    pub invulnerability_duration: f32,
    pub is_dead: bool,
}

impl Health {
    pub fn new(max: f32, invuln_duration: f32) -> Self {
        Self {
            current: max,
            max,
            invulnerability_timer: 0.0,
            invulnerability_duration: invuln_duration,
            is_dead: false,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.invulnerability_timer > 0.0 {
            self.invulnerability_timer = (self.invulnerability_timer - dt).max(0.0);
        }
    }

    /// Applies attack damage according to the GDD 6.2 damage taxonomy.
    /// Returns true if damage was registered (not blocked by invulnerability).
    pub fn take_damage(
        &mut self,
        amount: f32,
        damage_type: DamageType,
        flame: Option<&mut FlameEnergy>,
    ) -> bool {
        if self.invulnerability_timer > 0.0 || self.is_dead {
            return false;
        }

        match damage_type {
            DamageType::HealthDamage => {
                self.current = (self.current - amount).max(0.0);
                self.invulnerability_timer = self.invulnerability_duration;
            }
            DamageType::FlameDamage => {
                if let Some(flame_ref) = flame {
                    let _ = flame_ref.consume(amount);
                }
            }
            DamageType::DualDamage => {
                self.current = (self.current - amount * 0.75).max(0.0);
                if let Some(flame_ref) = flame {
                    let _ = flame_ref.consume(amount * 0.75);
                }
                self.invulnerability_timer = self.invulnerability_duration;
            }
        }

        if self.current <= 0.0 {
            self.is_dead = true;
        }

        true
    }

    pub fn heal(&mut self, amount: f32) {
        if !self.is_dead {
            self.current = (self.current + amount).min(self.max);
        }
    }
}

/// Axis-Aligned Bounding Box (AABB) for 2D combat hitboxes and hurtboxes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AABB2D {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl AABB2D {
    pub fn from_center_size(cx: f32, cy: f32, w: f32, h: f32) -> Self {
        let hw = w * 0.5;
        let hh = h * 0.5;
        Self {
            min_x: cx - hw,
            min_y: cy - hh,
            max_x: cx + hw,
            max_y: cy + hh,
        }
    }

    #[inline]
    pub fn intersects(&self, other: &AABB2D) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }
}
