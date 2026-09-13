#[allow(dead_code)]
pub mod seed;

use serde::{Deserialize, Serialize};
use crate::models::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[allow(dead_code)]
pub struct DatabaseImport {
    #[serde(default)]
    pub ingredients: Vec<Ingredient>,
    #[serde(default)]
    pub dishes: Vec<Dish>,
}

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

    pub fn generate_unique_ingredient_id(&self, prefix: &str) -> String {
        let mut i = 1;
        loop {
            let candidate = format!("{}_{}", prefix, i);
            if !self.ingredients.iter().any(|ing| ing.id == candidate) {
                return candidate;
            }
            i += 1;
        }
    }

    pub fn generate_unique_dish_id(&self, prefix: &str) -> String {
        let mut i = 1;
        loop {
            let candidate = format!("{}_{}", prefix, i);
            if !self.dishes.iter().any(|d| d.id == candidate) {
                return candidate;
            }
            i += 1;
        }
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

    /// Afegeix o actualitza ingredients i plats des d'una base de dades sense esborrar menús ni diaris
    #[allow(dead_code)]
    pub fn merge_database(&mut self, db: DatabaseImport) -> (usize, usize) {
        let mut ing_count = 0;
        let mut dish_count = 0;

        for incoming_ing in db.ingredients {
            if let Some(existing) = self.ingredients.iter_mut().find(|i| {
                i.id == incoming_ing.id || (!incoming_ing.name.trim().is_empty() && i.name.trim().eq_ignore_ascii_case(incoming_ing.name.trim()))
            }) {
                *existing = incoming_ing;
            } else {
                self.ingredients.push(incoming_ing);
            }
            ing_count += 1;
        }

        for incoming_dish in db.dishes {
            if let Some(existing) = self.dishes.iter_mut().find(|d| {
                d.id == incoming_dish.id || (!incoming_dish.name.trim().is_empty() && d.name.trim().eq_ignore_ascii_case(incoming_dish.name.trim()))
            }) {
                *existing = incoming_dish;
            } else {
                self.dishes.push(incoming_dish);
            }
            dish_count += 1;
        }

        (ing_count, dish_count)
    }

    /// Parsea un JSON flexible (pot ser {"ingredients": [...], "dishes": [...]}, array directe, estat complet, etc.) i fusiona els aliments/plats
    pub fn merge_database_from_json(&mut self, json_str: &str) -> Result<(usize, usize), String> {
        let val: serde_json::Value = serde_json::from_str(json_str).map_err(|e| format!("Error en el format JSON: {}", e))?;
        
        let mut ing_count = 0;
        let mut dish_count = 0;

        match val {
            serde_json::Value::Object(map) => {
                // Comprova claus d'ingredients (català, anglès o castellà)
                if let Some(ing_val) = map.get("ingredients").or_else(|| map.get("aliments")).or_else(|| map.get("alimentos")) {
                    if let Ok(ings) = serde_json::from_value::<Vec<Ingredient>>(ing_val.clone()) {
                        for incoming_ing in ings {
                            if let Some(existing) = self.ingredients.iter_mut().find(|i| {
                                (!incoming_ing.id.is_empty() && i.id == incoming_ing.id) 
                                || (!incoming_ing.name.trim().is_empty() && i.name.trim().eq_ignore_ascii_case(incoming_ing.name.trim()))
                            }) {
                                *existing = incoming_ing;
                            } else {
                                self.ingredients.push(incoming_ing);
                            }
                            ing_count += 1;
                        }
                    }
                }

                // Comprova claus de plats / receptes (català, anglès o castellà)
                if let Some(dish_val) = map.get("dishes").or_else(|| map.get("plats")).or_else(|| map.get("platos")).or_else(|| map.get("receptes")) {
                    if let Ok(dishes) = serde_json::from_value::<Vec<Dish>>(dish_val.clone()) {
                        for incoming_dish in dishes {
                            if let Some(existing) = self.dishes.iter_mut().find(|d| {
                                (!incoming_dish.id.is_empty() && d.id == incoming_dish.id)
                                || (!incoming_dish.name.trim().is_empty() && d.name.trim().eq_ignore_ascii_case(incoming_dish.name.trim()))
                            }) {
                                *existing = incoming_dish;
                            } else {
                                self.dishes.push(incoming_dish);
                            }
                            dish_count += 1;
                        }
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                // Comprova si és directament una llista d'ingredients o de plats
                if let Ok(ings) = serde_json::from_value::<Vec<Ingredient>>(serde_json::Value::Array(arr.clone())) {
                    for incoming_ing in ings {
                        if let Some(existing) = self.ingredients.iter_mut().find(|i| {
                            (!incoming_ing.id.is_empty() && i.id == incoming_ing.id)
                            || (!incoming_ing.name.trim().is_empty() && i.name.trim().eq_ignore_ascii_case(incoming_ing.name.trim()))
                        }) {
                            *existing = incoming_ing;
                        } else {
                            self.ingredients.push(incoming_ing);
                        }
                        ing_count += 1;
                    }
                } else if let Ok(dishes) = serde_json::from_value::<Vec<Dish>>(serde_json::Value::Array(arr)) {
                    for incoming_dish in dishes {
                        if let Some(existing) = self.dishes.iter_mut().find(|d| {
                            (!incoming_dish.id.is_empty() && d.id == incoming_dish.id)
                            || (!incoming_dish.name.trim().is_empty() && d.name.trim().eq_ignore_ascii_case(incoming_dish.name.trim()))
                        }) {
                            *existing = incoming_dish;
                        } else {
                            self.dishes.push(incoming_dish);
                        }
                        dish_count += 1;
                    }
                }
            }
            _ => return Err("El format ha de ser un document JSON (objecte o llista)".to_string()),
        }

        if ing_count == 0 && dish_count == 0 {
            Err("No s'ha trobat cap aliment ni cap plat vàlid en el JSON proporcionat".to_string())
        } else {
            Ok((ing_count, dish_count))
        }
    }
}

pub fn save_state_to_json(state: &AppState) -> Result<String, String> {
    serde_json::to_string_pretty(state).map_err(|e| e.to_string())
}

pub fn load_state_from_json(json_str: &str) -> Result<AppState, String> {
    serde_json::from_str(json_str).map_err(|e| e.to_string())
}

