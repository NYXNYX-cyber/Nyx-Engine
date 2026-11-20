use super::damage::{AABB2D, DamageType, Health};
use serde::{Deserialize, Serialize};

/// 5 distinct enemy archetypes defined in GDD Section 6.1:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyArchetype {
    /// Crawler: Ground-dwelling creature patrolling floors and ledges.
    Crawler,
    /// Wisp: Floating ethereal spirit harassing from unexpected angles.
    Wisp,
    /// Hollow: Humanoid darkness entity with deliberate strike patterns.
    Hollow,
    /// Ashwing: Airborne bat/gargoyle predator swooping down from ceilings.
    Ashwing,
    /// Devourer: Heavy brute behemoth exerting immense pressure.
    Devourer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIState {
    Idle,
    Patrol,
    Pursue,
    AttackWindup,
    Attacking,
    Cooldown,
}

/// The Enemy entity component encapsulating behaviour, archetype, and stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyEntity {
    pub id: u32,
    pub archetype: EnemyArchetype,
    pub health: Health,
    pub damage_type: DamageType,
    pub attack_damage: f32,
    pub ember_shards_reward: u32,

    // Position & Bounds
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub width: f32,
    pub height: f32,

    // AI parameters
    pub state: AIState,
    pub detection_radius: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub cooldown_timer: f32,
    pub patrol_origin_x: f32,
    pub patrol_distance: f32,
    pub move_speed: f32,
    pub facing_right: bool,
}

impl EnemyEntity {
    pub fn new(id: u32, archetype: EnemyArchetype, x: f32, y: f32) -> Self {
        match archetype {
            EnemyArchetype::Crawler => Self {
                id,
                archetype,
                health: Health::new(45.0, 0.2),
                damage_type: DamageType::HealthDamage,
                attack_damage: 15.0,
                ember_shards_reward: 15,
                x,
                y,
                vx: 0.0,
                vy: 0.0,
                width: 24.0,
                height: 18.0,
                state: AIState::Patrol,
                detection_radius: 180.0,
                attack_range: 28.0,
                attack_cooldown: 1.2,
                cooldown_timer: 0.0,
                patrol_origin_x: x,
                patrol_distance: 120.0,
                move_speed: 65.0,
                facing_right: true,
            },
            EnemyArchetype::Wisp => Self {
                id,
                archetype,
                health: Health::new(30.0, 0.1),
                damage_type: DamageType::FlameDamage, // Wisps feed on Flame!
                attack_damage: 20.0,
                ember_shards_reward: 25,
                x,
                y,
                vx: 0.0,
                vy: 0.0,
                width: 16.0,
                height: 16.0,
                state: AIState::Patrol,
                detection_radius: 240.0,
                attack_range: 35.0,
                attack_cooldown: 1.8,
                cooldown_timer: 0.0,
                patrol_origin_x: x,
                patrol_distance: 90.0,
                move_speed: 110.0,
                facing_right: true,
            },
            EnemyArchetype::Hollow => Self {
                id,
                archetype,
                health: Health::new(90.0, 0.3),
                damage_type: DamageType::DualDamage, // Lethal dual strike
                attack_damage: 25.0,
                ember_shards_reward: 45,
                x,
                y,
                vx: 0.0,
                vy: 0.0,
                width: 22.0,
                height: 44.0,
                state: AIState::Idle,
                detection_radius: 200.0,
                attack_range: 40.0,
                attack_cooldown: 1.5,
                cooldown_timer: 0.0,
                patrol_origin_x: x,
                patrol_distance: 80.0,
                move_speed: 90.0,
                facing_right: false,
            },
            EnemyArchetype::Ashwing => Self {
                id,
                archetype,
                health: Health::new(55.0, 0.2),
                damage_type: DamageType::HealthDamage,
                attack_damage: 22.0,
                ember_shards_reward: 35,
                x,
                y,
                vx: 0.0,
                vy: 0.0,
                width: 28.0,
                height: 20.0,
                state: AIState::Patrol,
                detection_radius: 280.0,
                attack_range: 45.0,
                attack_cooldown: 2.0,
                cooldown_timer: 0.0,
                patrol_origin_x: x,
                patrol_distance: 160.0,
                move_speed: 140.0,
                facing_right: true,
            },
            EnemyArchetype::Devourer => Self {
                id,
                archetype,
                health: Health::new(260.0, 0.4),
                damage_type: DamageType::DualDamage,
                attack_damage: 50.0,
                ember_shards_reward: 120,
                x,
                y,
                vx: 0.0,
                vy: 0.0,
                width: 52.0,
                height: 64.0,
                state: AIState::Idle,
                detection_radius: 220.0,
                attack_range: 65.0,
                attack_cooldown: 2.5,
                cooldown_timer: 0.0,
                patrol_origin_x: x,
                patrol_distance: 60.0,
                move_speed: 45.0,
                facing_right: false,
            },
        }
    }

    pub fn bounding_box(&self) -> AABB2D {
        AABB2D::from_center_size(self.x, self.y, self.width, self.height)
    }

    /// Step AI behavior based on player position.
    pub fn update_ai(&mut self, player_x: f32, player_y: f32, dt: f32) {
        if self.health.is_dead {
            return;
        }

        self.health.tick(dt);
        if self.cooldown_timer > 0.0 {
            self.cooldown_timer = (self.cooldown_timer - dt).max(0.0);
        }

        let dist_x = player_x - self.x;
        let dist_y = player_y - self.y;
        let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();

        match self.state {
            AIState::Idle | AIState::Patrol => {
                if dist <= self.detection_radius {
                    self.state = AIState::Pursue;
                } else if self.state == AIState::Patrol {
                    // Back and forth patrol along origin
                    let offset = self.x - self.patrol_origin_x;
                    if offset.abs() >= self.patrol_distance {
                        self.facing_right = offset < 0.0;
                    }
                    let dir = if self.facing_right { 1.0 } else { -1.0 };
                    self.vx = dir * self.move_speed;
                    self.x += self.vx * dt;
                }
            }
            AIState::Pursue => {
                self.facing_right = dist_x > 0.0;
                if dist <= self.attack_range && self.cooldown_timer <= 0.0 {
                    self.state = AIState::AttackWindup;
                } else if dist > self.detection_radius * 1.5 {
                    self.state = AIState::Patrol;
                } else {
                    let dir = if self.facing_right { 1.0 } else { -1.0 };
                    self.vx = dir * self.move_speed * 1.25;
                    self.x += self.vx * dt;

                    // Flying enemies adjust Y towards player
                    if matches!(self.archetype, EnemyArchetype::Wisp | EnemyArchetype::Ashwing) {
                        let y_dir = if dist_y > 0.0 { 1.0 } else { -1.0 };
                        self.vy = y_dir * (self.move_speed * 0.7);
                        self.y += self.vy * dt;
                    }
                }
            }
            AIState::AttackWindup => {
                self.vx = 0.0;
                self.state = AIState::Attacking;
            }
            AIState::Attacking => {
                self.cooldown_timer = self.attack_cooldown;
                self.state = AIState::Cooldown;
            }
            AIState::Cooldown => {
                if self.cooldown_timer <= 0.0 {
                    self.state = AIState::Pursue;
                }
            }
        }
    }
}
