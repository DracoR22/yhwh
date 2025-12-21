use std::{fs::{self, File}, io::Write};

use crate::{common::create_info::{GameObjectCreateInfo, LevelCreateInfo}, engine::GameData};

pub fn save_level(game_data: &GameData) {
    let mut game_object_create_infos: Vec<GameObjectCreateInfo> = Vec::new();

    for game_object in game_data.game_objects.iter() {
      game_object_create_infos.push(game_object.get_create_info(&game_data.asset_manager));
    }

    let level_create_info = LevelCreateInfo {
      name: "test".to_string(),
      game_objects: game_object_create_infos
    };

    let json = serde_json::to_string_pretty(&level_create_info).unwrap();

    match File::create(String::from("res/scenes/") + &level_create_info.name + ".json").unwrap().write_all(json.as_bytes()) {
      Ok(_msg) => { println!("Level saved!") },
      Err(err) => { println!("Could not save level. Error: {}", err) }
    }
}

#[derive(Debug)]
pub enum LoadLevelError {
  ReadError,
  SerdeError
}

pub fn load_level() -> Result<LevelCreateInfo, LoadLevelError> {
  let path = "res/scenes/test.json";
  let json = fs::read_to_string(path).map_err(|_| LoadLevelError::ReadError)?;

  let level: LevelCreateInfo = serde_json::from_str(&json).map_err(|_| LoadLevelError::SerdeError)?;

  Ok(level)
}