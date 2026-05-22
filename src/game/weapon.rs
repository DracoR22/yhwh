pub struct WeaponAnimations {
    pub walk: usize,
    pub idle: usize,
    pub idle_to_ads: usize,
    pub sprint: usize,
    pub sprint_end: usize,
    pub sprint_start: usize,
    pub sprint_back: usize,
    pub fire: usize,
    pub draw: usize,
    pub initial_draw: usize,
    pub holster: usize,
    pub reload: usize,
    pub reload_ads: usize,
    pub reload_empty: usize,
    pub reload_empty_ads: usize,
    pub ads_fire: usize,
    pub ads_idle: usize,
    pub ads_to_idle: usize,
    pub cqb_sprint: usize,
    pub cqb_sprint_end: usize,
    pub cqb_sprint_start: usize
}

#[derive(PartialEq, Eq)]
pub enum WeaponAction {
    Idle,
    Fire, 
    Walk,
    InitialDraw
}

pub struct WeaponInfo {
    pub object_id: usize,
    pub has: bool,
    pub equipped: bool,
    pub mag_size: i32,
    pub max_ammo: i32,
    pub animations: WeaponAnimations
}

pub struct WeaponManager {
    pub desert_eagle_info: WeaponInfo
}

impl WeaponManager {
    pub fn new(object_id: usize) -> Self {
        let desert_eagle_info = WeaponInfo {
            has: false,
            equipped: false,
            mag_size: 9,
            max_ammo: 99,
            object_id, 
            animations: WeaponAnimations { 
                ads_fire: 0,
                ads_idle: 1,
                ads_to_idle: 2,
                cqb_sprint: 3,
                cqb_sprint_end: 4,
                cqb_sprint_start: 5,
                draw: 6,
                fire: 7,
                holster: 8,
                idle: 9,
                idle_to_ads: 10,
                initial_draw: 11,
                reload: 12,
                reload_ads: 13,
                reload_empty: 14,
                reload_empty_ads: 15,
                sprint: 16,
                sprint_back: 17,
                sprint_end: 18,
                sprint_start: 19,
                walk: 20
            }
        };

        Self { 
            desert_eagle_info
         }
    }
}