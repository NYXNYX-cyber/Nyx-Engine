use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FacingDirection {
    Left,
    Right,
}

/// 2D Platformer physics and movement controller with Double Jump, Coyote Time, and Jump Buffering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformerMovement2D {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub width: f32,
    pub height: f32,

    // Tuning constants
    pub run_speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    pub air_control: f32,
    pub gravity: f32,
    pub max_fall_speed: f32,

    // Jump parameters
    pub jump_force: f32,
    pub double_jump_force: f32,
    pub jumps_remaining: u8,
    pub max_jumps: u8, // Set to 2 for Jump + Double Jump (GDD 4.1)

    // State & Feeling Buffers
    pub is_grounded: bool,
    pub facing: FacingDirection,
    pub coyote_timer: f32,
    pub coyote_duration: f32,
    pub jump_buffer_timer: f32,
    pub jump_buffer_duration: f32,
}

impl PlatformerMovement2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            width: 20.0,
            height: 36.0,

            run_speed: 240.0,
            acceleration: 1400.0,
            friction: 1200.0,
            air_control: 800.0,
            gravity: 980.0,
            max_fall_speed: 650.0,

            jump_force: 420.0,
            double_jump_force: 380.0,
            jumps_remaining: 2,
            max_jumps: 2,

            is_grounded: false,
            facing: FacingDirection::Right,
            coyote_timer: 0.0,
            coyote_duration: 0.12, // 120ms coyote time
            jump_buffer_timer: 0.0,
            jump_buffer_duration: 0.10, // 100ms jump buffer
        }
    }

    /// Feeds movement input from player (-1.0 to 1.0) and updates horizontal speed.
    pub fn apply_horizontal_input(&mut self, move_axis: f32, dt: f32) {
        if move_axis.abs() > 0.05 {
            let accel = if self.is_grounded { self.acceleration } else { self.air_control };
            let target_vx = move_axis.signum() * self.run_speed;
            if (target_vx - self.vx).abs() > 0.1 {
                self.vx += move_axis.signum() * accel * dt;
                if move_axis > 0.0 {
                    self.vx = self.vx.min(self.run_speed);
                    self.facing = FacingDirection::Right;
                } else {
                    self.vx = self.vx.max(-self.run_speed);
                    self.facing = FacingDirection::Left;
                }
            }
        } else {
            // Apply friction deceleration
            if self.is_grounded {
                let decel = self.friction * dt;
                if self.vx.abs() <= decel {
                    self.vx = 0.0;
                } else {
                    self.vx -= self.vx.signum() * decel;
                }
            }
        }
    }

    /// Queues a jump press (jump buffer).
    pub fn press_jump(&mut self) {
        self.jump_buffer_timer = self.jump_buffer_duration;
    }

    /// Applies vertical gravity and physics step.
    pub fn step_physics(&mut self, dt: f32) {
        // Decrement buffer timers
        if self.coyote_timer > 0.0 {
            self.coyote_timer = (self.coyote_timer - dt).max(0.0);
        }
        if self.jump_buffer_timer > 0.0 {
            self.jump_buffer_timer = (self.jump_buffer_timer - dt).max(0.0);
        }

        // Check if jump should execute from buffer
        if self.jump_buffer_timer > 0.0 {
            if self.is_grounded || self.coyote_timer > 0.0 {
                // First Jump from ground or coyote ledge
                self.vy = self.jump_force;
                self.is_grounded = false;
                self.coyote_timer = 0.0;
                self.jumps_remaining = self.max_jumps.saturating_sub(1);
                self.jump_buffer_timer = 0.0;
            } else if self.jumps_remaining > 0 {
                // Double Jump in mid-air (GDD 4.1)
                self.vy = self.double_jump_force;
                self.jumps_remaining = self.jumps_remaining.saturating_sub(1);
                self.jump_buffer_timer = 0.0;
            }
        }

        // Apply downward gravity when in the air
        if !self.is_grounded {
            self.vy -= self.gravity * dt;
            self.vy = self.vy.max(-self.max_fall_speed);
        }

        // Advance positions
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }

    /// Call when vertical raycast / collision detection resolves floor contact.
    pub fn on_landed(&mut self, ground_y: f32) {
        self.y = ground_y;
        self.vy = 0.0;
        self.is_grounded = true;
        self.jumps_remaining = self.max_jumps;
        self.coyote_timer = self.coyote_duration;
    }

    /// Call when player walks off a ledge into the air.
    pub fn on_left_ground(&mut self) {
        self.is_grounded = false;
        // Coyote timer allows grace jump for coyote_duration
    }
}
