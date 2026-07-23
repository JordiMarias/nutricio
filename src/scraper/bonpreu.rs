use regex::Regex;
use serde_json::Value;
use crate::models::*;

#[derive(Debug, Clone)]
pub struct ScrapedProduct {
    pub name: String,
    pub brand: Option<String>,
    pub price: Option<f64>,
    pub pack_weight_g: Option<f64>,
    pub nutrition_100g: NutritionalInfo,
    pub ingredients_text: Option<String>,
    pub nova_group: NovaGroup,
    pub glycemic_index: u8,
    pub source_url: String,
}


pub fn parse_bonpreu_html(html: &str, url: &str) -> Result<ScrapedProduct, String> {
    let mut name = "Producte Bonpreu".to_string();
    let mut brand: Option<String> = None;
    let mut price: Option<f64> = None;
    let mut pack_weight_g: Option<f64> = None;
    let mut nutrition_100g = NutritionalInfo::default();
    let mut ingredients_text: Option<String> = None;

    // 1. Extract JSON-LD
    let re_json_ld = Regex::new(r#"(?s)<script[^>]*type=['"]application/ld\+json['"][^>]*>(.*?)</script>"#).map_err(|e| e.to_string())?;
    for cap in re_json_ld.captures_iter(html) {
        if let Some(json_str) = cap.get(1) {
            if let Ok(val) = serde_json::from_str::<Value>(json_str.as_str()) {
                if val.get("@type").and_then(|t| t.as_str()) == Some("Product") {
                    if let Some(n) = val.get("name").and_then(|v| v.as_str()) {
                        name = n.to_string();
                    }
                    if let Some(b) = val.get("brand").and_then(|v| v.as_str()) {
                        brand = Some(b.to_string());
                    }
                    if let Some(offers) = val.get("offers") {
                        if let Some(p_str) = offers.get("price").and_then(|v| v.as_str()) {
                            price = p_str.parse::<f64>().ok();
                        } else if let Some(p_num) = offers.get("price").and_then(|v| v.as_f64()) {
                            price = Some(p_num);
                        }
                    }
                    if let Some(size) = val.get("size").and_then(|v| v.as_str()) {
                        pack_weight_g = parse_weight_g(size);
                    }
                    if pack_weight_g.is_none() {
                        if let Some(desc) = val.get("description").and_then(|v| v.as_str()) {
                            pack_weight_g = parse_weight_g(desc);
                        }
                    }
                    if pack_weight_g.is_none() {
                        pack_weight_g = parse_weight_g(&name);
                    }
                }
            }
        }
    }


    // 2. Extract Ingredients
    let re_ing = Regex::new(r"(?i)Ingredients\s*</h2>\s*([^<]+)").ok();
    if let Some(re) = re_ing {
        if let Some(cap) = re.captures(html) {
            ingredients_text = Some(cap[1].trim().to_string());
        }
    }
    if ingredients_text.is_none() {
        let re_ing_alt = Regex::new(r"(?i)INGREDIENTS:\s*([^<\n]+)").ok();
        if let Some(re) = re_ing_alt {
            if let Some(cap) = re.captures(html) {
                ingredients_text = Some(cap[1].trim().to_string());
            }
        }
    }

    // 3. Extract Nutrition Table values
    // Look for Kcal
    if let Some(re) = Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*kcal").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.kcal = cap[1].replace(',', ".").parse::<f64>().unwrap_or(0.0);
        }
    }

    // Greixos
    if let Some(re) = Regex::new(r"(?i)Greixos\s*</td>\s*<td[^>]*>\s*([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.fat_g = parse_val_g(&cap[1]);
        }
    } else if let Some(re) = Regex::new(r"(?i)Greixos[^<]*?([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.fat_g = parse_val_g(&cap[1]);
        }
    }

    // Greixos saturats
    if let Some(re) = Regex::new(r"(?i)saturats\s*</td>\s*<td[^>]*>\s*([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.saturated_fat_g = parse_val_g(&cap[1]);
        }
    } else if let Some(re) = Regex::new(r"(?i)saturats[^<]*?([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.saturated_fat_g = parse_val_g(&cap[1]);
        }
    }

    // Hidrats de carboni
    if let Some(re) = Regex::new(r"(?i)Hidrats de carboni\s*</td>\s*<td[^>]*>\s*([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.carbs_g = parse_val_g(&cap[1]);
        }
    } else if let Some(re) = Regex::new(r"(?i)Hidrats de carboni[^<]*?([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.carbs_g = parse_val_g(&cap[1]);
        }
    }

    // Sucres
    if let Some(re) = Regex::new(r"(?i)sucres\s*</td>\s*<td[^>]*>\s*([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.sugars_g = parse_val_g(&cap[1]);
        }
    } else if let Some(re) = Regex::new(r"(?i)sucres[^<]*?([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.sugars_g = parse_val_g(&cap[1]);
        }
    }

    // Fibra alimentària
    if let Some(re) = Regex::new(r"(?i)Fibra alimentària\s*</td>\s*<td[^>]*>\s*([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.fiber_g = parse_val_g(&cap[1]);
        }
    } else if let Some(re) = Regex::new(r"(?i)Fibra[^<]*?([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.fiber_g = parse_val_g(&cap[1]);
        }
    }

    // Proteïnes
    if let Some(re) = Regex::new(r"(?i)Proteïnes\s*</td>\s*<td[^>]*>\s*([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.protein_g = parse_val_g(&cap[1]);
        }
    } else if let Some(re) = Regex::new(r"(?i)Proteïnes[^<]*?([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.protein_g = parse_val_g(&cap[1]);
        }
    }

    // Sal
    if let Some(re) = Regex::new(r"(?i)Sal\s*</td>\s*<td[^>]*>\s*([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.salt_g = parse_val_g(&cap[1]);
        }
    } else if let Some(re) = Regex::new(r"(?i)Sal[^<]*?([<\d.,]+)\s*g").ok() {
        if let Some(cap) = re.captures(html) {
            nutrition_100g.salt_g = parse_val_g(&cap[1]);
        }
    }


    // Calculate marginal price per 100g if price and pack_weight are present
    if let (Some(p), Some(w)) = (price, pack_weight_g) {
        if w > 0.0 {
            nutrition_100g.price_euro = (p / w) * 100.0;
        }
    }

    let nova_group = detect_nova_group(&name, ingredients_text.as_deref());
    let glycemic_index = estimate_glycemic_index(&name, nutrition_100g.carbs_g, ingredients_text.as_deref());

    Ok(ScrapedProduct {
        name,
        brand,
        price,
        pack_weight_g,
        nutrition_100g,
        ingredients_text,
        nova_group,
        glycemic_index,
        source_url: url.to_string(),
    })

}

fn parse_val_g(text: &str) -> f64 {
    let clean = text.replace('<', "").replace(',', ".").trim().to_string();
    clean.parse::<f64>().unwrap_or(0.0)
}

fn parse_weight_g(text: &str) -> Option<f64> {
    let text_lower = text.to_lowercase();

    // 1. Kg or Litres (1kg = 1000g, 1L = 1000ml = 1000g)
    if let Ok(re) = Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*(?:kg|l|litre|litres)\b") {
        if let Some(cap) = re.captures(&text_lower) {
            if let Ok(val) = cap[1].replace(',', ".").parse::<f64>() {
                return Some(val * 1000.0);
            }
        }
    }

    // 2. Centilitres (1cl = 10ml = 10g)
    if let Ok(re) = Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*cl\b") {
        if let Some(cap) = re.captures(&text_lower) {
            if let Ok(val) = cap[1].replace(',', ".").parse::<f64>() {
                return Some(val * 10.0);
            }
        }
    }

    // 3. Millilitres or Grams (1ml = 1g)
    if let Ok(re) = Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*(?:ml|g|gram|grams|grames)\b") {
        if let Some(cap) = re.captures(&text_lower) {
            if let Ok(val) = cap[1].replace(',', ".").parse::<f64>() {
                return Some(val);
            }
        }
    }

    None
}


#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_and_parse_bonpreu_url(url: &str) -> Result<ScrapedProduct, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().map_err(|e| format!("Error en carregar URL: {}", e))?;
    let html = resp.text().map_err(|e| format!("Error en llegir resposta HTTP: {}", e))?;

    parse_bonpreu_html(&html, url)
}
