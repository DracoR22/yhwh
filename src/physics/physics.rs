use std::collections::HashMap;

use rapier3d::{control::KinematicCharacterController, prelude::*};

use crate::{common::{constants::FIXED_DELTA_TIME, errors::CharacterControllerError}, utils::unique_id};

struct CharacterController {
    shape: SharedShape,
    position: Isometry<f32>,
    kinematic_character_controller: KinematicCharacterController,
}

pub struct Physics {
    gravity: nalgebra::Vector3<f32>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
   // query_pipeline: QueryPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver, 
    accumulated_time: f32,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    
    rigid_handles: HashMap<usize, RigidBodyHandle>,
    character_controllers: HashMap<usize, CharacterController>
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
            rigid_handles,
            character_controllers: HashMap::new()
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

    //   if let Some(ball_rigid_handle) = self.rigid_handles.get(&0) {
    //     if let Some(body) = self.rigid_body_set.get(*ball_rigid_handle) {
    //     println!("Ball altitude: {}", body.translation().y);
    //    }
    //   }
     }

     pub fn create_character_controller(&mut self) -> usize {
       let character_controller = CharacterController { 
        position: Isometry::translation(0.0, 1.0, 0.0),
        shape: SharedShape::capsule_y(0.9, 0.4),
        kinematic_character_controller: KinematicCharacterController::default()
       };

       let physics_id = unique_id::next_id();
       self.character_controllers.insert(physics_id, character_controller);

       physics_id
     }

    //  pub fn character_controller_exists(&self) -> Result<(), >{

    //  }

     pub fn move_character_controller(&self, delta_time: std::time::Duration, direction: cgmath::Vector3<f32>, controller_id: usize) -> Result<(), CharacterControllerError> {
        let character_controller = self.character_controllers.get(&controller_id).ok_or(CharacterControllerError::ControllerNotFound)?;

        // let controller_handle = self.rigid_handles.get(&controller_id).ok_or(CharacterControllerError::ControllerHandleNotFound)?;
        // let controller_body = self.rigid_body_set.get(*controller_handle).ok_or(CharacterControllerError::ControllerBodyNotFound)?;

        // character_controller.kinematic_character_controller.move_shape(
        //     delta_time.as_secs_f32(), 
        //     &self.physics_pipeline,
        //     &*character_controller.shape,
        //     &character_controller.position,
        //     Vector::new(direction.x, direction.y, direction.z),
        //     |_| {},
        // );

        Ok(())
     }
}