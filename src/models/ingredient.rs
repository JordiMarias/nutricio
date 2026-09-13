use serde::{Deserialize, Serialize};
use super::glycemic::{calculate_glycemic_load, estimate_glycemic_index, GlycemicLevel};
use super::nova::NovaGroup;
use super::nutrition::NutritionalInfo;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UnitType {
    #[serde(alias = "Per100ml")]
    Per100g,
    PerUnit { grams_per_unit: f64 },
}

impl Default for UnitType {
    fn default() -> Self {
        UnitType::Per100g
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ingredient {
    pub id: String,
    pub name: String,
    pub brand: Option<String>,
    pub source_url: Option<String>,
    pub unit_type: UnitType,
    pub per_unit_nutrition: NutritionalInfo, // Valors per 100g o per unitat
    pub price_per_pack: Option<f64>,
    pub pack_weight_g: Option<f64>,
    pub nova_group: Option<NovaGroup>,
    pub glycemic_index: Option<u8>, // Índex Glucèmic (0..=100)
    pub ingredients_text: Option<String>,
}

impl Ingredient {
    #[allow(dead_code)]
    pub fn new(id: impl Into<String>, name: impl Into<String>, nutrition_100g: NutritionalInfo) -> Self {
        let name_str = name.into();
        let estimated_ig = estimate_glycemic_index(&name_str, nutrition_100g.carbs_g, None);

        Self {
            id: id.into(),
            name: name_str,
            brand: None,
            source_url: None,
            unit_type: UnitType::Per100g,
            per_unit_nutrition: nutrition_100g,
            price_per_pack: None,
            pack_weight_g: None,
            nova_group: Some(NovaGroup::Group1Unprocessed),
            glycemic_index: Some(estimated_ig),
            ingredients_text: None,
        }
    }

    pub fn get_glycemic_index(&self) -> u8 {
        self.glycemic_index.unwrap_or_else(|| {
            estimate_glycemic_index(&self.name, self.per_unit_nutrition.carbs_g, self.ingredients_text.as_deref())
        })
    }

    pub fn get_glycemic_level(&self) -> GlycemicLevel {
        GlycemicLevel::from_ig(self.get_glycemic_index())
    }

    pub fn calculate_nutrition(&self, quantity: f64) -> NutritionalInfo {
        match self.unit_type {
            UnitType::Per100g => {
                let factor = quantity / 100.0;
                self.per_unit_nutrition.scale(factor)
            }
            UnitType::PerUnit { .. } => {
                self.per_unit_nutrition.scale(quantity)
            }
        }
    }

    pub fn calculate_glycemic_load(&self, quantity: f64) -> f64 {
        let nut = self.calculate_nutrition(quantity);
        let ig = self.get_glycemic_index();
        calculate_glycemic_load(ig, nut.carbs_g)
    }
}
