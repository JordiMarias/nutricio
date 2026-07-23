use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlycemicLevel {
    Low = 1,    // IG <= 55
    Medium = 2, // IG 56..69
    High = 3,   // IG >= 70
}

impl GlycemicLevel {
    pub fn from_ig(ig: u8) -> Self {
        if ig <= 55 {
            GlycemicLevel::Low
        } else if ig <= 69 {
            GlycemicLevel::Medium
        } else {
            GlycemicLevel::High
        }
    }

    #[allow(dead_code)]
    pub fn name_ca(&self) -> &'static str {
        match self {
            GlycemicLevel::Low => "IG Baix (<=55)",
            GlycemicLevel::Medium => "IG Mitjà (56-69)",
            GlycemicLevel::High => "IG Alt (>=70)",
        }
    }

    #[allow(dead_code)]
    pub fn short_name_ca(&self) -> &'static str {
        match self {
            GlycemicLevel::Low => "IG Baix",
            GlycemicLevel::Medium => "IG Mitjà",
            GlycemicLevel::High => "IG Alt",
        }
    }

    pub fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            GlycemicLevel::Low => (46, 125, 50),     // Green
            GlycemicLevel::Medium => (245, 124, 0), // Amber / Yellow
            GlycemicLevel::High => (198, 40, 40),   // Red
        }
    }
}

pub fn estimate_glycemic_index(name: &str, carbs_100g: f64, ingredients_text: Option<&str>) -> u8 {
    // 1. If negligible carbs, IG is 0
    if carbs_100g < 1.0 {
        return 0;
    }

    let name_lower = name.to_lowercase();
    let ing_lower = ingredients_text.unwrap_or("").to_lowercase();

    // 2. High GI foods (>= 70)
    let high_keywords = [
        "sucre", "pa blanc", "patata", "galet", "pastís", "caramel",
        "dolç", "mel", "xarop", "sirop", "maltodextrin", "corn flake",
        "arròs blanc", "farina refinada", "refresc", "beguda sucrosa"
    ];
    if high_keywords.iter().any(|&k| name_lower.contains(k) || ing_lower.contains(k)) {
        return 75;
    }

    // 3. Low GI foods (<= 55)
    let low_keywords = [
        "llenti", "cigr", "monget", "soja", "pèso", "peso", "llegum",
        "poma", "pera", "taronja", "kiwi", "fresa", "maduixa", "alvocat",
        "llet", "iogurt", "frut", "nous", "ametll", "avellan",
        "civada", "integral"
    ];
    if low_keywords.iter().any(|&k| name_lower.contains(k) || ing_lower.contains(k)) {
        return 35;
    }

    // 4. Medium GI foods (56..69)
    let medium_keywords = [
        "basmati", "quinoa", "plàtan", "platan", "raïm", "raim", "cuscús", "cuscus"
    ];
    if medium_keywords.iter().any(|&k| name_lower.contains(k) || ing_lower.contains(k)) {
        return 55;
    }


    // Default estimate based on sugar ratio
    if carbs_100g > 0.0 {
        50
    } else {
        0
    }
}

pub fn calculate_glycemic_load(ig: u8, carbs_g: f64) -> f64 {
    (ig as f64 * carbs_g) / 100.0
}
