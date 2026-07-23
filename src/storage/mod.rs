pub mod seed;

use serde::{Deserialize, Serialize};
use crate::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub ingredients: Vec<Ingredient>,
    pub dishes: Vec<Dish>,
    pub weekly_menu: WeeklyMenu,
    pub goals: NutritionalGoals,
}

impl Default for AppState {
    fn default() -> Self {
        Self::empty()
    }
}

impl AppState {
    pub fn empty() -> Self {
        Self {
            ingredients: Vec::new(),
            dishes: Vec::new(),
            weekly_menu: WeeklyMenu::default(),
            goals: NutritionalGoals::default(),
        }
    }

    pub fn with_seed_data() -> Self {
        Self {
            ingredients: seed::get_seed_ingredients(),
            dishes: seed::get_seed_dishes(),
            weekly_menu: seed::get_seed_weekly_menu(),
            goals: NutritionalGoals::default(),
        }
    }
}

pub fn save_state_to_json(state: &AppState) -> Result<String, String> {
    serde_json::to_string_pretty(state).map_err(|e| e.to_string())
}

pub fn load_state_from_json(json_str: &str) -> Result<AppState, String> {
    serde_json::from_str(json_str).map_err(|e| e.to_string())
}
