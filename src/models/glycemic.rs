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
    if carbs_100g < 0.5 {
        return 0;
    }

    let name_lower = name.to_lowercase();
    let ing_lower = ingredients_text.unwrap_or("").to_lowercase();

    // Specific high-impact products
    if name_lower.contains("beguda d'arròs") || name_lower.contains("beguda d'arros") || name_lower.contains("beguda arros") {
        return 85;
    }
    if name_lower.contains("cervesa") || ing_lower.contains("malt d'ordi") {
        return 70;
    }

    // 2. High GI foods (>= 70)
    let high_keywords = [
        "sucre", "pa blanc", "patata", "galet", "pastís", "caramel",
        "dolç", "mel", "xarop", "sirop", "maltodextrin", "corn flake",
        "arròs blanc", "farina refinada", "refresc", "beguda sucrosa"
    ];
    if high_keywords.iter().any(|&k| name_lower.contains(k) || ing_lower.contains(k)) {
        return 75;
    }

    // 3. Very low GI non-starchy vegetables (<= 15)
    let very_low_keywords = [
        "bròcoli", "brocoli", "amanida", "enciam", "espinac", "escarola", "carbassó", "carbasso", "pebrot"
    ];
    if very_low_keywords.iter().any(|&k| name_lower.contains(k) || ing_lower.contains(k)) {
        return 15;
    }

    // 4. Low GI foods (<= 55)
    let low_keywords = [
        "llenti", "cigr", "monget", "soja", "pèso", "peso", "llegum",
        "poma", "pera", "taronja", "kiwi", "fresa", "maduixa", "alvocat",
        "llet", "iogurt", "frut", "nous", "ametll", "avellan",
        "civada", "integral", "sègol", "segol", "fajol", "quinoa", "plàtan", "platan", "tomàquet", "tomaquet"
    ];
    if low_keywords.iter().any(|&k| name_lower.contains(k) || ing_lower.contains(k)) {
        if name_lower.contains("kiwi") || name_lower.contains("plàtan") || name_lower.contains("platan") {
            return 52;
        }
        if name_lower.contains("quinoa") {
            return 53;
        }
        if name_lower.contains("fajol") {
            return 50;
        }
        if name_lower.contains("basmati") {
            return 45;
        }
        return 35;
    }

    // 5. Medium GI foods (56..69)
    let medium_keywords = [
        "raïm", "raim", "cuscús", "cuscus"
    ];
    if medium_keywords.iter().any(|&k| name_lower.contains(k) || ing_lower.contains(k)) {
        return 58;
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
