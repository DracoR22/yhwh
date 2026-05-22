use crate::{asset_manager::AssetManager, common::create_info::{AnimatedGameObjectCreateInfo, DoorObjectCreateInfo, GameObjectCreateInfo, LightObjectCreateInfo, MeshNodeCreateInfo}, objects::{animated_game_object::AnimatedGameObject, door_object::DoorObject, game_object::GameObject, light_object::LightObject}, utils::json::load_level};

pub struct Scene {
    pub game_objects: Vec<GameObject>,
    pub animated_game_objects: Vec<AnimatedGameObject>,
    pub door_objects: Vec<DoorObject>,
    pub lights: Vec<LightObject>
}

impl Scene {
    pub fn new(asset_manager: &AssetManager) -> Self {
        let mut game_objects = Vec::<GameObject>::new();
        let mut animated_game_objects = Vec::<AnimatedGameObject>::new();  
        let mut door_objects = Vec::<DoorObject>::new();
        let mut lights = Vec::<LightObject>::new();

        let level = load_level().expect("Could not load level!!");

        for create_info in level.game_objects {
            game_objects.push(GameObject::new(&create_info, &asset_manager));
        }

        // todo: serialize animated game objects into json instead
        //  let glock_create_info = GameObjectCreateInfo {
        //     model_name: "untitled2".to_string(),
        //     position: [10.0, 2.0, 0.0],
        //     rotation: [1.0, 1.0, 1.0],
        //     size: [0.08, 0.08, 0.08],
        //     tex_scale: [1.0, 1.0],
        //     shadows: false,
        //     mesh_rendering_info: vec![]
        // };

          let glock_create_info2 = AnimatedGameObjectCreateInfo {
            model_name: "untitled2".to_string(),
            position: [10.0, 2.0, 10.0],
            rotation: [0.0, 0.0, 0.0],
            size: [0.08, 0.08, 0.08],
            tex_scale: [1.0, 1.0],
            loop_anim: true,
            mesh_rendering_info: vec![]
        };

        //animated_game_objects.push(AnimatedGameObject::new(&glock_create_info, &asset_manager));
        animated_game_objects.push(AnimatedGameObject::new(&glock_create_info2, &asset_manager));

        for create_info in level.door_objects {
            door_objects.push(DoorObject::new(&create_info, asset_manager));
        }

        for create_info in level.lights {
            lights.push(LightObject::new(&create_info));
        }

        Self {
            game_objects,
            animated_game_objects,
            lights,
            door_objects
        }
    }

    pub fn add_game_object(&mut self, create_info: &GameObjectCreateInfo, asset_manager: &AssetManager) {
        self.game_objects.push(GameObject::new(&create_info, asset_manager));
    }

    pub fn add_animated_game_object(&mut self, create_info: &AnimatedGameObjectCreateInfo, asset_manager: &AssetManager) -> usize {
        let animated_game_object = AnimatedGameObject::new(create_info, asset_manager);
        let id = animated_game_object.id;
        self.animated_game_objects.push(animated_game_object);

        id
    }

    pub fn remove_game_object_by_id(&mut self, id: usize) {
        self.game_objects.retain(|g| g.id != id);
    }
}

// Lights
impl Scene {
    pub fn add_light(&mut self, create_info: &LightObjectCreateInfo) {
        self.lights.push(LightObject::new(&create_info));
    }
}

// doors
impl Scene {
    pub fn add_door_object(&mut self, create_info: &DoorObjectCreateInfo, asset_manager: &AssetManager) {
        self.door_objects.push(DoorObject::new(create_info, asset_manager));
    }
}