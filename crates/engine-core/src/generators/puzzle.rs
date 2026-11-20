use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PuzzleType {
    /// Brazier Gate: Requires lighting all designated braziers in the sector to open a massive door.
    BrazierGate { braziers_required: u8, braziers_lit: u8 },
    /// Dark Bramble: Organic shadowy thorns blocking a hallway, burned away by Flame Wave or Flame Burst.
    DarkBramble { burn_health: f32, max_burn_health: f32 },
    /// Frozen Barrier: Glacial wall melted by focused Fireball or Flame Energy contact.
    FrozenBarrier { melt_progress: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlamePuzzleObstacle {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub puzzle_type: PuzzleType,
    pub is_cleared: bool,
}

impl FlamePuzzleObstacle {
    pub fn new_bramble(id: u32, x: f32, y: f32, burn_health: f32) -> Self {
        Self {
            id,
            x,
            y,
            width: 32.0,
            height: 64.0,
            puzzle_type: PuzzleType::DarkBramble {
                burn_health,
                max_burn_health: burn_health,
            },
            is_cleared: false,
        }
    }

    pub fn new_brazier_gate(id: u32, x: f32, y: f32, braziers_required: u8) -> Self {
        Self {
            id,
            x,
            y,
            width: 24.0,
            height: 96.0,
            puzzle_type: PuzzleType::BrazierGate {
                braziers_required,
                braziers_lit: 0,
            },
            is_cleared: false,
        }
    }

    /// Feeds fire attack damage into the obstacle. Returns true if cleared.
    pub fn apply_fire_hit(&mut self, fire_damage: f32) -> bool {
        if self.is_cleared {
            return true;
        }

        match &mut self.puzzle_type {
            PuzzleType::DarkBramble { burn_health, .. } => {
                *burn_health -= fire_damage;
                if *burn_health <= 0.0 {
                    self.is_cleared = true;
                }
            }
            PuzzleType::FrozenBarrier { melt_progress } => {
                *melt_progress += fire_damage * 0.02;
                if *melt_progress >= 1.0 {
                    self.is_cleared = true;
                }
            }
            PuzzleType::BrazierGate { braziers_required, braziers_lit } => {
                *braziers_lit = (*braziers_lit + 1).min(*braziers_required);
                if *braziers_lit >= *braziers_required {
                    self.is_cleared = true;
                }
            }
        }

        self.is_cleared
    }
}
