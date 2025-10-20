use std::collections::HashMap;

use rapier3d::prelude::*;

use crate::common::constants::FIXED_DELTA_TIME;

pub struct Physics {
    gravity: nalgebra::Vector3<f32>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver, 
    accumulated_time: f32,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    rigid_handles: HashMap<usize, RigidBodyHandle>
}

impl Physics {
    pub fn new() -> Self {
       let mut rigid_body_set = RigidBodySet::new();
       let mut collider_set = ColliderSet::new();
       let mut rigid_handles: HashMap<usize, RigidBodyHandle> = HashMap::new();

       /* Create the ground. */
       let collider = ColliderBuilder::cuboid(100.0, 0.1, 100.0).build();
       collider_set.insert(collider);

       let rigid_body = RigidBodyBuilder::dynamic()
        .translation(vector![0.0, 10.0, 0.0])
        .build();
       let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
       let ball_body_handle = rigid_body_set.insert(rigid_body);
       collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);

       rigid_handles.insert(0, ball_body_handle);


        Self {
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            rigid_body_set,
            collider_set,
            accumulated_time: 0.0,
            rigid_handles
        }
    }

    pub fn step_simulation(&mut self, delta_time: std::time::Duration) {
        self.accumulated_time += delta_time.as_secs_f32();

        while self.accumulated_time >= FIXED_DELTA_TIME {
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
            &(),
            &()
        );

        self.accumulated_time -= FIXED_DELTA_TIME;
      }

      if let Some(ball_rigid_handle) = self.rigid_handles.get(&0) {
        if let Some(body) = self.rigid_body_set.get(*ball_rigid_handle) {
        println!("Ball altitude: {}", body.translation().y);
       }
      }
      
     }
}