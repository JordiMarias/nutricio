use serde::{Deserialize, Serialize};
use super::glycemic::GlycemicLevel;
use super::ingredient::Ingredient;

use super::nova::NovaGroup;
use super::nutrition::NutritionalInfo;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DishItem {
    pub ingredient_id: String,
    pub quantity: f64, // Nombre de grams o d'unitats segons l'ingredient
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dish {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub items: Vec<DishItem>,
    pub servings: f64,
}

impl Dish {
    #[allow(dead_code)]
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            items: Vec::new(),
            servings: 1.0,
        }
    }

    pub fn calculate_total_nutrition(&self, ingredients_db: &[Ingredient]) -> NutritionalInfo {
        let mut total = NutritionalInfo::zero();
        for item in &self.items {
            if let Some(ing) = ingredients_db.iter().find(|i| i.id == item.ingredient_id) {
                let item_nut = ing.calculate_nutrition(item.quantity);
                total.add(&item_nut);
            }
        }
        if self.servings > 0.0 && self.servings != 1.0 {
            total.scale(1.0 / self.servings)
        } else {
            total
        }
    }

    pub fn nova_breakdown(&self, ingredients_db: &[Ingredient]) -> std::collections::HashMap<NovaGroup, f64> {
        let mut map = std::collections::HashMap::new();
        map.insert(NovaGroup::Group1Unprocessed, 0.0);
        map.insert(NovaGroup::Group2ProcessedIngredient, 0.0);
        map.insert(NovaGroup::Group3Processed, 0.0);
        map.insert(NovaGroup::Group4UltraProcessed, 0.0);

        let serving_factor = if self.servings > 0.0 && self.servings != 1.0 {
            1.0 / self.servings
        } else {
            1.0
        };

        for item in &self.items {
            if let Some(ing) = ingredients_db.iter().find(|i| i.id == item.ingredient_id) {
                let grp = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                let nut = ing.calculate_nutrition(item.quantity * serving_factor);
                *map.entry(grp).or_insert(0.0) += nut.kcal;
            }
        }
        map
    }

    pub fn derived_nova_group(&self, ingredients_db: &[Ingredient]) -> NovaGroup {
        let breakdown = self.nova_breakdown(ingredients_db);
        breakdown
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(grp, _)| grp)
            .unwrap_or(NovaGroup::Group1Unprocessed)
    }

    pub fn calculate_glycemic_load(&self, ingredients_db: &[Ingredient]) -> f64 {
        let mut total_cg = 0.0;
        for item in &self.items {
            if let Some(ing) = ingredients_db.iter().find(|i| i.id == item.ingredient_id) {
                total_cg += ing.calculate_glycemic_load(item.quantity);
            }
        }
        if self.servings > 0.0 && self.servings != 1.0 {
            total_cg / self.servings
        } else {
            total_cg
        }
    }

    pub fn derived_glycemic_index(&self, ingredients_db: &[Ingredient]) -> u8 {
        let nut = self.calculate_total_nutrition(ingredients_db);
        if nut.carbs_g <= 0.0 {
            return 0;
        }
        let cg = self.calculate_glycemic_load(ingredients_db);
        let derived_ig = ((cg * 100.0) / nut.carbs_g).round();
        (derived_ig as u8).min(100)
    }

    pub fn derived_glycemic_level(&self, ingredients_db: &[Ingredient]) -> GlycemicLevel {
        GlycemicLevel::from_ig(self.derived_glycemic_index(ingredients_db))
    }
}
