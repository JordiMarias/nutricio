use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::dish::Dish;
use super::ingredient::Ingredient;
use super::nova::NovaGroup;
use super::nutrition::NutritionalInfo;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MealType {
    Breakfast,  // Esmorzar (Matí)
    MidMorning, // Mig matí
    Lunch,      // Dinar (Matí-migdia)
    Afternoon,  // Berenar
    Dinner,     // Sopar (Tarda-Vespre)
}

impl MealType {
    pub fn name_ca(&self) -> &'static str {
        match self {
            MealType::Breakfast => "Esmorzar (Matí)",
            MealType::MidMorning => "Mig matí",
            MealType::Lunch => "Dinar (Matí-migdia)",
            MealType::Afternoon => "Berenar",
            MealType::Dinner => "Sopar (Tarda-Vespre)",
        }
    }

    pub fn all() -> Vec<MealType> {
        vec![
            MealType::Breakfast,
            MealType::MidMorning,
            MealType::Lunch,
            MealType::Afternoon,
            MealType::Dinner,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MealEntry {
    pub id: String,
    pub item_id: String, // ID d'ingredient o de plat
    pub is_dish: bool,
    pub quantity: f64,   // Nombre de grams o racions
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DailyMenu {
    pub day_name: String,
    pub meals: HashMap<MealType, Vec<MealEntry>>,
}

impl DailyMenu {
    pub fn new(day_name: impl Into<String>) -> Self {
        let mut meals = HashMap::new();
        for m in MealType::all() {
            meals.insert(m, Vec::new());
        }
        Self {
            day_name: day_name.into(),
            meals,
        }
    }

    pub fn calculate_total_nutrition(
        &self,
        ingredients_db: &[Ingredient],
        dishes_db: &[Dish],
    ) -> NutritionalInfo {
        let mut total = NutritionalInfo::zero();
        for entries in self.meals.values() {
            for entry in entries {
                if entry.is_dish {
                    if let Some(dish) = dishes_db.iter().find(|d| d.id == entry.item_id) {
                        let dish_nut = dish.calculate_total_nutrition(ingredients_db);
                        total.add(&dish_nut.scale(entry.quantity));
                    }
                } else {
                    if let Some(ing) = ingredients_db.iter().find(|i| i.id == entry.item_id) {
                        let ing_nut = ing.calculate_nutrition(entry.quantity);
                        total.add(&ing_nut);
                    }
                }
            }
        }
        total
    }

    pub fn calculate_total_glycemic_load(
        &self,
        ingredients_db: &[Ingredient],
        dishes_db: &[Dish],
    ) -> f64 {
        let mut total_cg = 0.0;
        for entries in self.meals.values() {
            for entry in entries {
                if entry.is_dish {
                    if let Some(dish) = dishes_db.iter().find(|d| d.id == entry.item_id) {
                        total_cg += dish.calculate_glycemic_load(ingredients_db) * entry.quantity;
                    }
                } else {
                    if let Some(ing) = ingredients_db.iter().find(|i| i.id == entry.item_id) {
                        total_cg += ing.calculate_glycemic_load(entry.quantity);
                    }
                }
            }
        }
        total_cg
    }

    pub fn nova_breakdown(
        &self,
        ingredients_db: &[Ingredient],
        dishes_db: &[Dish],
    ) -> HashMap<NovaGroup, f64> {
        let mut map = HashMap::new();
        map.insert(NovaGroup::Group1Unprocessed, 0.0);
        map.insert(NovaGroup::Group2ProcessedIngredient, 0.0);
        map.insert(NovaGroup::Group3Processed, 0.0);
        map.insert(NovaGroup::Group4UltraProcessed, 0.0);

        for entries in self.meals.values() {
            for entry in entries {
                let (group, kcal) = if entry.is_dish {
                    if let Some(dish) = dishes_db.iter().find(|d| d.id == entry.item_id) {
                        let grp = dish.derived_nova_group(ingredients_db);
                        let nut = dish.calculate_total_nutrition(ingredients_db).scale(entry.quantity);
                        (grp, nut.kcal)
                    } else {
                        continue;
                    }
                } else {
                    if let Some(ing) = ingredients_db.iter().find(|i| i.id == entry.item_id) {
                        let grp = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                        let nut = ing.calculate_nutrition(entry.quantity);
                        (grp, nut.kcal)
                    } else {
                        continue;
                    }
                };

                *map.entry(group).or_insert(0.0) += kcal;
            }
        }
        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeeklyMenu {
    pub name: String,
    pub days: Vec<DailyMenu>,
}

impl Default for WeeklyMenu {
    fn default() -> Self {
        let days_names = ["Dilluns", "Dimarts", "Dimecres", "Dijous", "Divendres", "Dissabte", "Diumenge"];
        let days = days_names.iter().map(|name| DailyMenu::new(*name)).collect();
        Self {
            name: "Menú Setmanal Tipus".to_string(),
            days,
        }
    }
}
