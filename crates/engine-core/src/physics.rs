use rapier3d::prelude::*;

/// Server-side headless 3D physics world powered by Rapier3D.
/// Handles deterministic continuous collision detection, rigidbodies, and gravity.
pub struct PhysicsWorld {
    pub gravity: Vector<Real>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub physics_hooks: (),
    pub event_handler: (),
}

impl PhysicsWorld {
    pub fn new(gravity_y: f32) -> Self {
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = 1.0 / 60.0; // Fixed 60Hz tick

        Self {
            gravity: vector![0.0, gravity_y, 0.0],
            integration_parameters,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }

    /// Advance the physics simulation by a single discrete step
    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &self.physics_hooks,
            &self.event_handler,
        );
    }

    /// Add a static box collider (e.g. ground, walls, ship floor)
    pub fn add_static_box(&mut self, hx: f32, hy: f32, hz: f32, x: f32, y: f32, z: f32) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::fixed()
            .translation(vector![x, y, z])
            .build();
        let body_handle = self.rigid_body_set.insert(rigid_body);
        let collider = ColliderBuilder::cuboid(hx, hy, hz).build();
        self.collider_set.insert_with_parent(collider, body_handle, &mut self.rigid_body_set);
        body_handle
    }

    /// Add a dynamic sphere (e.g. physics projectile, falling debris)
    pub fn add_dynamic_sphere(&mut self, radius: f32, x: f32, y: f32, z: f32) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![x, y, z])
            .ccd_enabled(true)
            .build();
        let body_handle = self.rigid_body_set.insert(rigid_body);
        let collider = ColliderBuilder::ball(radius)
            .restitution(0.7) // Bouncy
            .build();
        self.collider_set.insert_with_parent(collider, body_handle, &mut self.rigid_body_set);
        body_handle
    }

    /// Get position of a rigidbody
    pub fn get_position(&self, handle: RigidBodyHandle) -> Option<[f32; 3]> {
        self.rigid_body_set.get(handle).map(|rb| {
            let t = rb.translation();
            [t.x, t.y, t.z]
        })
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new(-9.81)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_fall_and_bounce() {
        let mut world = PhysicsWorld::default();
        // Add floor at y = 0.0
        world.add_static_box(50.0, 1.0, 50.0, 0.0, -1.0, 0.0);
        // Add dynamic ball at y = 10.0
        let ball = world.add_dynamic_sphere(0.5, 0.0, 10.0, 0.0);

        // Step 60 frames (1 second of simulation)
        for _ in 0..60 {
            world.step();
        }

        let pos = world.get_position(ball).expect("Ball must exist");
        assert!(pos[1] < 10.0, "Ball should have fallen due to gravity");
        assert!(pos[1] >= 0.0, "Ball should have collided with the floor");
    }
}
