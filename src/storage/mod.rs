#[allow(dead_code)]
pub mod seed;

use serde::{Deserialize, Serialize};
use crate::models::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppState {
    pub ingredients: Vec<Ingredient>,
    pub dishes: Vec<Dish>,
    pub weekly_menu: WeeklyMenu,
    pub goals: NutritionalGoals,
    #[serde(default)]
    pub daily_journal: Vec<DailyLog>,
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
            daily_journal: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.ingredients.is_empty()
            && self.dishes.is_empty()
            && self.weekly_menu.days.iter().all(|d| d.meals.values().all(|entries| entries.is_empty()))
            && self.daily_journal.iter().all(|l| l.daily_menu.meals.values().all(|entries| entries.is_empty()))
    }

    #[allow(dead_code)]
    pub fn get_or_create_daily_log(&mut self, date_str: &str, day_label: &str) -> &mut DailyLog {
        if !self.daily_journal.iter().any(|l| l.date == date_str) {
            self.daily_journal.push(DailyLog::new(date_str, day_label));
        }
        self.daily_journal.iter_mut().find(|l| l.date == date_str).unwrap()
    }

    #[allow(dead_code)]
    pub fn with_seed_data() -> Self {
        Self {
            ingredients: seed::get_seed_ingredients(),
            dishes: seed::get_seed_dishes(),
            weekly_menu: seed::get_seed_weekly_menu(),
            goals: NutritionalGoals::default(),
            daily_journal: Vec::new(),
        }
    }
}

pub fn save_state_to_json(state: &AppState) -> Result<String, String> {
    serde_json::to_string_pretty(state).map_err(|e| e.to_string())
}

pub fn load_state_from_json(json_str: &str) -> Result<AppState, String> {
    serde_json::from_str(json_str).map_err(|e| e.to_string())
}
