use rapier3d::prelude::*;

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
    physics_hooks: Box<dyn PhysicsHooks>,
    event_handler: Box<dyn EventHandler>,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet 
}

impl Physics {
    pub fn new() -> Self {
       let mut rigid_body_set = RigidBodySet::new();
       let mut collider_set = ColliderSet::new();

       /* Create the ground. */
       let collider = ColliderBuilder::cuboid(100.0, 0.1, 100.0).build();
       collider_set.insert(collider);

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
            physics_hooks: Box::new(()),
            event_handler: Box::new(()),
            rigid_body_set,
            collider_set
        }
    }

    pub fn step_simulation(&mut self) {
        for _ in 0..200 {
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
            self.physics_hooks.as_ref(),
            self.event_handler.as_ref(),
        );
      }
    }
}