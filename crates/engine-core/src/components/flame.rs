use serde::{Deserialize, Serialize};

/// State of the Last Flame carried by the protagonist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlameState {
    /// 80% - 100%: Burning bright, maximum light radius and combat power.
    Radiant,
    /// 35% - 79%: Stable flame, standard light radius.
    Stable,
    /// 10% - 34%: Flickering weak flame, reduced visibility, abilities restricted.
    Flickering,
    /// 0.1% - 9%: Critical ember, near extinction, heavy darkness encroaching.
    Critical,
    /// 0%: Flame is completely dead. Triggers FireflyRescue or Death.
    Extinguished,
}

/// The core resource component representing the player's lifeline and power source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameEnergy {
    pub current: f32,
    pub max: f32,
    /// Passive burn drain per second when traversing deep darkness without environmental light.
    pub drain_rate: f32,
    /// Base light radius emitted by the flame in world units (pixels or meters).
    pub base_light_radius: f32,
}

impl FlameEnergy {
    pub fn new(max: f32, drain_rate: f32, base_light_radius: f32) -> Self {
        Self {
            current: max,
            max,
            drain_rate,
            base_light_radius,
        }
    }

    pub fn state(&self) -> FlameState {
        let pct = (self.current / self.max).clamp(0.0, 1.0);
        if self.current <= 0.001 {
            FlameState::Extinguished
        } else if pct >= 0.80 {
            FlameState::Radiant
        } else if pct >= 0.35 {
            FlameState::Stable
        } else if pct >= 0.10 {
            FlameState::Flickering
        } else {
            FlameState::Critical
        }
    }

    /// Ticks passive flame burn over time. When standing in a safe lighted zone, drain is halted.
    pub fn tick(&mut self, dt: f32, in_safe_light: bool) -> FlameState {
        if !in_safe_light && self.current > 0.0 {
            self.current = (self.current - self.drain_rate * dt).max(0.0);
        }
        self.state()
    }

    /// Consumes flame energy for abilities or interactions. Returns false if insufficient energy.
    pub fn consume(&mut self, amount: f32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    /// Restores flame energy up to maximum capacity.
    pub fn restore(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Calculates current dynamic illumination radius based on flame strength.
    pub fn dynamic_light_radius(&self) -> f32 {
        let pct = (self.current / self.max).clamp(0.0, 1.0);
        match self.state() {
            FlameState::Radiant => self.base_light_radius * (0.9 + 0.1 * pct),
            FlameState::Stable => self.base_light_radius * (0.7 + 0.25 * pct),
            FlameState::Flickering => self.base_light_radius * (0.4 + 0.3 * pct),
            FlameState::Critical => self.base_light_radius * 0.25,
            FlameState::Extinguished => 0.0,
        }
    }
}

/// The singular emergency lifeline defined in GDD Section 4.3:
/// When the flame is fully extinguished, the Firefly Rescue triggers once to rekindle the flame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FireflyRescue {
    pub has_lifeline: bool,
    pub is_active: bool,
    pub revive_percentage: f32,
    pub invulnerability_duration: f32,
    pub invulnerability_timer: f32,
}

impl FireflyRescue {
    pub fn new(revive_percentage: f32, invulnerability_duration: f32) -> Self {
        Self {
            has_lifeline: true,
            is_active: false,
            revive_percentage,
            invulnerability_duration,
            invulnerability_timer: 0.0,
        }
    }

    /// Attempts to rescue the player upon flame extinction.
    /// Returns true if rescue was granted; returns false if lifeline was already consumed (Permanent Death).
    pub fn trigger_rescue(&mut self, flame: &mut FlameEnergy) -> bool {
        if self.has_lifeline {
            self.has_lifeline = false;
            self.is_active = true;
            self.invulnerability_timer = self.invulnerability_duration;
            flame.current = flame.max * self.revive_percentage.clamp(0.1, 1.0);
            true
        } else {
            false
        }
    }

    /// Updates invulnerability countdown after rescue.
    pub fn tick(&mut self, dt: f32) {
        if self.invulnerability_timer > 0.0 {
            self.invulnerability_timer = (self.invulnerability_timer - dt).max(0.0);
            if self.invulnerability_timer == 0.0 {
                self.is_active = false;
            }
        }
    }
}

/// Checkpoint type defined in GDD Section 4.4:
/// - Store: Saves player position & progression state.
/// - Recharge: Refills flame energy and rekindles environmental warmth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointType {
    Store,
    Recharge,
    Dual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: u32,
    pub checkpoint_type: CheckpointType,
    pub x: f32,
    pub y: f32,
    pub is_ignited: bool,
    pub radius: f32,
}

impl Checkpoint {
    pub fn new(id: u32, checkpoint_type: CheckpointType, x: f32, y: f32, radius: f32) -> Self {
        Self {
            id,
            checkpoint_type,
            x,
            y,
            is_ignited: false,
            radius,
        }
    }

    /// Interacts with the checkpoint. Ignites the brazier and executes the designated effect.
    pub fn activate(&mut self, flame: &mut FlameEnergy) -> bool {
        self.is_ignited = true;
        match self.checkpoint_type {
            CheckpointType::Recharge => {
                flame.restore(flame.max);
                true
            }
            CheckpointType::Dual => {
                flame.restore(flame.max);
                true
            }
            CheckpointType::Store => true,
        }
    }
}
