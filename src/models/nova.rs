use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NovaGroup {
    #[serde(alias = "1", alias = "Group1", alias = "Group1Unprocessed")]
    Group1Unprocessed = 1,
    #[serde(alias = "2", alias = "Group2", alias = "Group2ProcessedIngredient")]
    Group2ProcessedIngredient = 2,
    #[serde(alias = "3", alias = "Group3", alias = "Group3Processed", alias = "Group3ProcessedFoods")]
    Group3Processed = 3,
    #[serde(alias = "4", alias = "Group4", alias = "Group4UltraProcessed")]
    Group4UltraProcessed = 4,
}

impl NovaGroup {
    pub fn name_ca(&self) -> &'static str {
        match self {
            NovaGroup::Group1Unprocessed => "NOVA 1: No Processat",
            NovaGroup::Group2ProcessedIngredient => "NOVA 2: Ingredient Culinari",
            NovaGroup::Group3Processed => "NOVA 3: Processat",
            NovaGroup::Group4UltraProcessed => "NOVA 4: Ultraprocessat",
        }
    }

    pub fn short_name_ca(&self) -> &'static str {
        match self {
            NovaGroup::Group1Unprocessed => "No Processat",
            NovaGroup::Group2ProcessedIngredient => "Ingredient Culinari",
            NovaGroup::Group3Processed => "Processat",
            NovaGroup::Group4UltraProcessed => "Ultraprocessat",
        }
    }

    #[allow(dead_code)]
    pub fn description_ca(&self) -> &'static str {
        match self {
            NovaGroup::Group1Unprocessed => "Aliments naturals, fressos o mínimament modificats (fruites, verdures, carns, peixos, ous, llegums, arròs, llet).",
            NovaGroup::Group2ProcessedIngredient => "Substàncies extretes de la natura utilitzades per cuinar (oli d'oliva, sal, sucre, vinagre, mantega).",
            NovaGroup::Group3Processed => "Aliments elaborats afegint ingredients del Grup 2 a aliments del Grup 1 (conserves, formatge, pa artesanal).",
            NovaGroup::Group4UltraProcessed => "Formulacions industrials amb additius, aromes, conservants, emulgents o greixos refinats/hidrogenats.",
        }
    }

    pub fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            NovaGroup::Group1Unprocessed => (46, 125, 50),     // Dark Green
            NovaGroup::Group2ProcessedIngredient => (245, 124, 0), // Amber / Gold
            NovaGroup::Group3Processed => (230, 81, 0),        // Deep Orange
            NovaGroup::Group4UltraProcessed => (198, 40, 40),   // Red
        }
    }
}

pub fn detect_nova_group(food_name: &str, ingredients_text: Option<&str>) -> NovaGroup {
    let name_lower = food_name.to_lowercase();
    let ing_text = ingredients_text.unwrap_or("").to_lowercase();

    // 1. Check for Ultra-processed indicators (Group 4)
    let ultra_keywords = [
        "aroma", "aromes", "conservant", "emulgent", "estabilitzant", "colorant",
        "edulcorant", "jarop", "sirop", "maltodextrina", "hidrogenat", "palma",
        "dextrosa", "proteïna de soja", "extracte de llevant", "glutamat",
        "sorbat", "benzoat", "nitrit", "nitrat", "antioxidant e-", "espesseïdor",
        "midó modificat", "mido modificat", "modificat", "goma garrofí", "goma garrofi",
        "goma xantana", "burger meat"
    ];

    if ultra_keywords.iter().any(|&k| ing_text.contains(k) || name_lower.contains(k)) {
        return NovaGroup::Group4UltraProcessed;
    }

    // Check for E-numbers in ingredients text or name
    let re_e_num = regex::Regex::new(r"(?i)\be-?\d{3,4}\b").unwrap();
    if re_e_num.is_match(&ing_text) || re_e_num.is_match(&name_lower) {
        return NovaGroup::Group4UltraProcessed;
    }

    // 2. Check for Processed Culinary Ingredients (Group 2)
    let group2_keywords = [
        "oli d'oliva", "oli de", "oli verge", "sal marina", "sal ", "sucre", "mantega", "vinagre"
    ];
    if group2_keywords.iter().any(|&k| name_lower.contains(k) || ing_text.contains(k)) {
        return NovaGroup::Group2ProcessedIngredient;
    }

    // 3. Check for Processed Foods (Group 3)
    let group3_keywords = [
        "conserva", "formatge", "mozarella", "mozzarella", "pa ", "pernil", "tonyina en oli", "sardines en oli", "embotit", "cervesa"
    ];
    if group3_keywords.iter().any(|&k| name_lower.contains(k) || ing_text.contains(k)) {
        return NovaGroup::Group3Processed;
    }

    // 4. Default to Group 1 (Unprocessed / Minimally processed)
    NovaGroup::Group1Unprocessed
}
