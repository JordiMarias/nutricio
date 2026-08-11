use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NutritionalInfo {
    pub kcal: f64,
    pub fat_g: f64,
    pub saturated_fat_g: f64,
    pub carbs_g: f64,
    pub sugars_g: f64,
    pub fiber_g: f64,
    pub protein_g: f64,
    pub salt_g: f64,
    pub price_euro: f64,
}

impl Default for NutritionalInfo {
    fn default() -> Self {
        Self {
            kcal: 0.0,
            fat_g: 0.0,
            saturated_fat_g: 0.0,
            carbs_g: 0.0,
            sugars_g: 0.0,
            fiber_g: 0.0,
            protein_g: 0.0,
            salt_g: 0.0,
            price_euro: 0.0,
        }
    }
}

impl NutritionalInfo {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn scale(&self, factor: f64) -> Self {
        Self {
            kcal: self.kcal * factor,
            fat_g: self.fat_g * factor,
            saturated_fat_g: self.saturated_fat_g * factor,
            carbs_g: self.carbs_g * factor,
            sugars_g: self.sugars_g * factor,
            fiber_g: self.fiber_g * factor,
            protein_g: self.protein_g * factor,
            salt_g: self.salt_g * factor,
            price_euro: self.price_euro * factor,
        }
    }

    pub fn add(&mut self, other: &Self) {
        self.kcal += other.kcal;
        self.fat_g += other.fat_g;
        self.saturated_fat_g += other.saturated_fat_g;
        self.carbs_g += other.carbs_g;
        self.sugars_g += other.sugars_g;
        self.fiber_g += other.fiber_g;
        self.protein_g += other.protein_g;
        self.salt_g += other.salt_g;
        self.price_euro += other.price_euro;
    }

    pub fn total_macro_grams(&self) -> f64 {
        self.fat_g + self.carbs_g + self.protein_g
    }

    pub fn fat_kcal(&self) -> f64 {
        self.fat_g * 9.0
    }

    pub fn protein_kcal(&self) -> f64 {
        self.protein_g * 4.0
    }

    pub fn carbs_kcal(&self) -> f64 {
        self.carbs_g * 4.0
    }

    pub fn total_macro_kcal(&self) -> f64 {
        self.fat_kcal() + self.protein_kcal() + self.carbs_kcal()
    }

    pub fn fat_pct(&self) -> f64 {
        let total_kcal = self.total_macro_kcal();
        if total_kcal > 0.0 { (self.fat_kcal() / total_kcal) * 100.0 } else { 0.0 }
    }

    pub fn carbs_pct(&self) -> f64 {
        let total_kcal = self.total_macro_kcal();
        if total_kcal > 0.0 { (self.carbs_kcal() / total_kcal) * 100.0 } else { 0.0 }
    }

    pub fn protein_pct(&self) -> f64 {
        let total_kcal = self.total_macro_kcal();
        if total_kcal > 0.0 { (self.protein_kcal() / total_kcal) * 100.0 } else { 0.0 }
    }

    pub fn sugar_pct_of_macros(&self) -> f64 {
        let total_kcal = self.total_macro_kcal();
        if total_kcal > 0.0 { ((self.sugars_g * 4.0) / total_kcal) * 100.0 } else { 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutritionalGoals {
    pub min_kcal: f64,
    pub max_kcal: f64,
    pub min_fat_pct: f64,
    pub max_fat_pct: f64,
    pub max_saturated_fat_g: f64,
    pub min_carbs_pct: f64,
    pub max_carbs_pct: f64,
    pub max_sugar_macro_pct: f64,
    pub min_fiber_g: f64,
    pub ideal_fiber_g: f64,
    pub min_protein_pct: f64,
    pub max_protein_pct: f64,
    pub min_protein_g: f64,
    pub max_salt_g: f64,
    pub max_ultraprocessed_pct: f64,
    pub max_glycemic_load: f64, // Objectiu màxim de Càrrega Glucèmica (idealment <= 100)
}

impl Default for NutritionalGoals {
    fn default() -> Self {
        Self {
            min_kcal: 2200.0,
            max_kcal: 2500.0,
            min_fat_pct: 20.0,
            max_fat_pct: 25.0,
            max_saturated_fat_g: 19.0,
            min_carbs_pct: 45.0,
            max_carbs_pct: 55.0,
            max_sugar_macro_pct: 10.0,
            min_fiber_g: 30.0,
            ideal_fiber_g: 40.0,
            min_protein_pct: 20.0,
            max_protein_pct: 25.0,
            min_protein_g: 130.0,
            max_salt_g: 5.0,
            max_ultraprocessed_pct: 10.0,
            max_glycemic_load: 100.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetStatus {
    Optimal,
    Warning,
    Critical,
}
