use egui::{Color32, ProgressBar, Ui};
use chrono::{Datelike, Local, NaiveDate};

use crate::exporter::generate_daily_journal_report_html;
use crate::models::*;
use crate::storage::AppState;
use crate::views::menu_planner::{draw_glycemic_badge, draw_nova_badge};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddItemCategory {
    Ingredient,
    Dish,
    Punctual,
}

pub struct DailyJournalView {
    pub selected_date: String,
    pub selected_meal: Option<MealType>,
    pub show_add_item_dialog: bool,
    pub show_edit_goals_modal: bool,
    pub show_copy_confirm_dialog: bool,
    pub copy_source_day_idx: usize,
    pub add_category: AddItemCategory,
    pub add_is_dish: bool,
    pub add_item_id: String,
    pub add_quantity: f64,
    pub picker_search: String,
    pub export_message: Option<String>,

    // Formulari d'aliment puntual
    pub punctual_name: String,
    pub punctual_brand: String,
    pub punctual_kcal: f64,
    pub punctual_fat: f64,
    pub punctual_sat_fat: f64,
    pub punctual_carbs: f64,
    pub punctual_sugar: f64,
    pub punctual_fiber: f64,
    pub punctual_protein: f64,
    pub punctual_salt: f64,
    pub punctual_price: f64,
    pub punctual_nova: NovaGroup,
    pub punctual_ig: u8,
    pub punctual_ingredients_text: String,
    pub punctual_json_paste: String,
    pub punctual_json_status: Option<String>,
}

impl Default for DailyJournalView {
    fn default() -> Self {
        Self {
            selected_date: get_today_date_str(),
            selected_meal: None,
            show_add_item_dialog: false,
            show_edit_goals_modal: false,
            show_copy_confirm_dialog: false,
            copy_source_day_idx: 0,
            add_category: AddItemCategory::Ingredient,
            add_is_dish: false,
            add_item_id: String::new(),
            add_quantity: 100.0,
            picker_search: String::new(),
            export_message: None,

            punctual_name: String::new(),
            punctual_brand: String::new(),
            punctual_kcal: 0.0,
            punctual_fat: 0.0,
            punctual_sat_fat: 0.0,
            punctual_carbs: 0.0,
            punctual_sugar: 0.0,
            punctual_fiber: 0.0,
            punctual_protein: 0.0,
            punctual_salt: 0.0,
            punctual_price: 0.0,
            punctual_nova: NovaGroup::Group1Unprocessed,
            punctual_ig: 50,
            punctual_ingredients_text: String::new(),
            punctual_json_paste: String::new(),
            punctual_json_status: None,
        }
    }
}

impl DailyJournalView {
    pub fn reset_punctual_form(&mut self) {
        self.punctual_name.clear();
        self.punctual_brand.clear();
        self.punctual_kcal = 0.0;
        self.punctual_fat = 0.0;
        self.punctual_sat_fat = 0.0;
        self.punctual_carbs = 0.0;
        self.punctual_sugar = 0.0;
        self.punctual_fiber = 0.0;
        self.punctual_protein = 0.0;
        self.punctual_salt = 0.0;
        self.punctual_price = 0.0;
        self.punctual_nova = NovaGroup::Group1Unprocessed;
        self.punctual_ig = 50;
        self.punctual_ingredients_text.clear();
        self.punctual_json_paste.clear();
        self.punctual_json_status = None;
    }

    pub fn load_punctual_from_json(&mut self) {
        let text = self.punctual_json_paste.trim();
        if text.is_empty() {
            self.punctual_json_status = Some("⚠️ Enganxa un text JSON abans d'importar.".to_string());
            return;
        }

        match parse_punctual_ingredient_json(text) {
            Ok(ing) => {
                let ig = ing.get_glycemic_index();
                self.punctual_name = ing.name;
                self.punctual_brand = ing.brand.unwrap_or_default();
                self.punctual_kcal = ing.per_unit_nutrition.kcal;
                self.punctual_fat = ing.per_unit_nutrition.fat_g;
                self.punctual_sat_fat = ing.per_unit_nutrition.saturated_fat_g;
                self.punctual_carbs = ing.per_unit_nutrition.carbs_g;
                self.punctual_sugar = ing.per_unit_nutrition.sugars_g;
                self.punctual_fiber = ing.per_unit_nutrition.fiber_g;
                self.punctual_protein = ing.per_unit_nutrition.protein_g;
                self.punctual_salt = ing.per_unit_nutrition.salt_g;
                self.punctual_price = ing.per_unit_nutrition.price_euro;
                self.punctual_nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                self.punctual_ig = ig;
                self.punctual_ingredients_text = ing.ingredients_text.unwrap_or_default();
                if let Some(w) = ing.pack_weight_g {
                    if w > 0.0 {
                        self.add_quantity = w;
                    }
                }
                self.punctual_json_status = Some("✅ Dades carregades correctament al formulari!".to_string());
            }
            Err(e) => {
                self.punctual_json_status = Some(format!("❌ Error en interpretar el JSON: {}", e));
            }
        }
    }
}

pub fn parse_punctual_ingredient_json(json_str: &str) -> Result<Ingredient, String> {
    let val: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Format JSON invàlid: {}", e))?;

    let target_obj = match &val {
        serde_json::Value::Object(map) => {
            if let Some(arr_val) = map.get("ingredients").or_else(|| map.get("aliments")).or_else(|| map.get("alimentos")).or_else(|| map.get("punctual")) {
                if let Some(first) = arr_val.as_array().and_then(|a| a.first()) {
                    first.clone()
                } else {
                    val.clone()
                }
            } else {
                val.clone()
            }
        }
        serde_json::Value::Array(arr) => {
            if let Some(first) = arr.first() {
                first.clone()
            } else {
                return Err("L'array JSON està buit".to_string());
            }
        }
        _ => return Err("El JSON ha de ser un objecte o una llista".to_string()),
    };

    // 1. Prova de deserialització directa amb Serde
    if let Ok(mut ing) = serde_json::from_value::<Ingredient>(target_obj.clone()) {
        if ing.id.is_empty() {
            ing.id = "punctual_temp".to_string();
        }
        return Ok(ing);
    }

    // 2. Parser flexible per a respostes d'agents IA
    if let serde_json::Value::Object(map) = target_obj {
        let name = map.get("name")
            .or_else(|| map.get("nom"))
            .or_else(|| map.get("title"))
            .or_else(|| map.get("plat"))
            .or_else(|| map.get("food"))
            .and_then(|v| v.as_str())
            .unwrap_or("Aliment Puntual")
            .to_string();

        let brand = map.get("brand")
            .or_else(|| map.get("marca"))
            .or_else(|| map.get("restaurant"))
            .or_else(|| map.get("lloc"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let nut_val = map.get("per_unit_nutrition")
            .or_else(|| map.get("nutrition"))
            .or_else(|| map.get("nutritional_info"))
            .or_else(|| map.get("nutricio"));

        let get_num = |keys: &[&str]| -> f64 {
            for k in keys {
                if let Some(n) = nut_val.and_then(|nv| nv.get(*k)).and_then(|v| v.as_f64()) {
                    return n;
                }
                if let Some(n) = map.get(*k).and_then(|v| v.as_f64()) {
                    return n;
                }
            }
            0.0
        };

        let kcal = get_num(&["kcal", "calories", "calorias", "energia"]);
        let fat_g = get_num(&["fat_g", "fat", "greixos", "grasas", "total_fat"]);
        let saturated_fat_g = get_num(&["saturated_fat_g", "saturated_fat", "greixos_saturats", "grasas_saturadas"]);
        let carbs_g = get_num(&["carbs_g", "carbs", "carbohydrates", "hidrats", "carbohidratos"]);
        let sugars_g = get_num(&["sugars_g", "sugars", "sugar", "sucres", "azucares"]);
        let fiber_g = get_num(&["fiber_g", "fiber", "fibra"]);
        let protein_g = get_num(&["protein_g", "protein", "proteina", "proteïna"]);
        let salt_g = get_num(&["salt_g", "salt", "sal"]);
        let price_euro = get_num(&["price_euro", "price", "preu", "precio"]);

        let nova_group = map.get("nova_group")
            .or_else(|| map.get("nova"))
            .and_then(|v| {
                if let Some(n) = v.as_u64() {
                    match n {
                        1 => Some(NovaGroup::Group1Unprocessed),
                        2 => Some(NovaGroup::Group2ProcessedIngredient),
                        3 => Some(NovaGroup::Group3Processed),
                        4 => Some(NovaGroup::Group4UltraProcessed),
                        _ => None,
                    }
                } else if let Some(s) = v.as_str() {
                    let sl = s.to_lowercase();
                    if sl.contains('4') || sl.contains("ultra") {
                        Some(NovaGroup::Group4UltraProcessed)
                    } else if sl.contains('3') || sl.contains("processedfoods") || (sl.contains("processed") && !sl.contains("ingredient") && !sl.contains("unprocessed")) {
                        Some(NovaGroup::Group3Processed)
                    } else if sl.contains('2') || sl.contains("ingredient") || sl.contains("culinary") {
                        Some(NovaGroup::Group2ProcessedIngredient)
                    } else if sl.contains('1') || sl.contains("unprocessed") {
                        Some(NovaGroup::Group1Unprocessed)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or(NovaGroup::Group1Unprocessed);

        let glycemic_index = map.get("glycemic_index")
            .or_else(|| map.get("gi"))
            .or_else(|| map.get("ig"))
            .or_else(|| map.get("index_glucemic"))
            .and_then(|v| v.as_u64())
            .map(|n| n.min(100) as u8);

        let ingredients_text = map.get("ingredients_text")
            .or_else(|| map.get("ingredients"))
            .or_else(|| map.get("text"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let pack_weight_g = map.get("pack_weight_g")
            .or_else(|| map.get("serving_size_g"))
            .or_else(|| map.get("serving_size"))
            .or_else(|| map.get("serving_g"))
            .or_else(|| map.get("portion_g"))
            .or_else(|| map.get("weight_g"))
            .or_else(|| map.get("weight"))
            .or_else(|| map.get("grams"))
            .or_else(|| map.get("quantitat"))
            .and_then(|v| v.as_f64());

        let nut = NutritionalInfo {
            kcal,
            fat_g,
            saturated_fat_g,
            carbs_g,
            sugars_g,
            fiber_g,
            protein_g,
            salt_g,
            price_euro,
        };

        Ok(Ingredient {
            id: "punctual_temp".to_string(),
            name,
            brand,
            source_url: None,
            unit_type: UnitType::Per100g,
            per_unit_nutrition: nut,
            price_per_pack: None,
            pack_weight_g,
            nova_group: Some(nova_group),
            glycemic_index,
            ingredients_text,
        })
    } else {
        Err("L'element no és un objecte JSON vàlid".to_string())
    }
}

pub fn get_today_date_str() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

pub fn format_catalan_date(date_str: &str) -> String {
    if let Ok(d) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let weekday_ca = match d.weekday() {
            chrono::Weekday::Mon => "Dilluns",
            chrono::Weekday::Tue => "Dimarts",
            chrono::Weekday::Wed => "Dimecres",
            chrono::Weekday::Thu => "Dijous",
            chrono::Weekday::Fri => "Divendres",
            chrono::Weekday::Sat => "Dissabte",
            chrono::Weekday::Sun => "Diumenge",
        };
        let month_ca = match d.month() {
            1 => "gener",
            2 => "febrer",
            3 => "març",
            4 => "abril",
            5 => "maig",
            6 => "juny",
            7 => "juliol",
            8 => "agost",
            9 => "setembre",
            10 => "octubre",
            11 => "novembre",
            12 => "desembre",
            _ => "",
        };
        format!("{}, {} de {} de {}", weekday_ca, d.day(), month_ca, d.year())
    } else {
        date_str.to_string()
    }
}

pub fn get_weekday_name_catalan(date_str: &str) -> &'static str {
    if let Ok(d) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        match d.weekday() {
            chrono::Weekday::Mon => "Dilluns",
            chrono::Weekday::Tue => "Dimarts",
            chrono::Weekday::Wed => "Dimecres",
            chrono::Weekday::Thu => "Dijous",
            chrono::Weekday::Fri => "Divendres",
            chrono::Weekday::Sat => "Dissabte",
            chrono::Weekday::Sun => "Diumenge",
        }
    } else {
        "Dilluns"
    }
}

pub fn offset_date(date_str: &str, days_delta: i64) -> String {
    if let Ok(d) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let new_d = if days_delta >= 0 {
            d.checked_add_signed(chrono::Duration::days(days_delta))
        } else {
            d.checked_sub_signed(chrono::Duration::days(-days_delta))
        };
        if let Some(nd) = new_d {
            return nd.format("%Y-%m-%d").to_string();
        }
    }
    date_str.to_string()
}

impl DailyJournalView {
    pub fn ui(&mut self, ui: &mut Ui, state: &mut AppState) {
        let today_str = get_today_date_str();
        if self.selected_date.is_empty() {
            self.selected_date = today_str.clone();
        }

        let is_today = self.selected_date == today_str;
        let weekday_name = get_weekday_name_catalan(&self.selected_date);
        let formatted_date = format_catalan_date(&self.selected_date);

        // Ensure daily log exists in state
        if !state.daily_journal.iter().any(|l| l.date == self.selected_date) {
            state.daily_journal.push(DailyLog::new(&self.selected_date, weekday_name));
        }
        let log_idx = state.daily_journal.iter().position(|l| l.date == self.selected_date).unwrap();

        ui.heading("📝 Seguiment Diari d'Àpats (Food Journal)");
        ui.label("Registra tot el que menges dia a dia per controlar la teva nutrició i despesa real.");
        ui.add_space(6.0);

        let screen_rect = ui.ctx().screen_rect();
        let is_mobile = screen_rect.width() < 750.0 || ui.available_width() < 750.0;
        let modal_width = if is_mobile { (screen_rect.width() - 20.0).max(280.0) } else { 550.0 };
        let modal_height = if is_mobile { (screen_rect.height() - 40.0).max(350.0) } else { 520.0 };

        let mut trigger_export = false;

        // SECTION 0: Date Navigation & Actions Bar
        ui.group(|ui| {
            if is_mobile {
                // Mobile layout for Date Navigation
                ui.horizontal(|ui| {
                    if ui.button("◀ Ahir").clicked() {
                        self.selected_date = offset_date(&self.selected_date, -1);
                    }
                    if ui.selectable_label(is_today, "📅 Avui").clicked() {
                        self.selected_date = today_str.clone();
                    }
                    if ui.button("Demà ▶").clicked() {
                        self.selected_date = offset_date(&self.selected_date, 1);
                    }
                });

                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.strong(format!("📅 {}", formatted_date));
                    if is_today {
                        ui.colored_label(Color32::from_rgb(34, 197, 94), "(Avui)");
                    }
                });

                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("📋 Copiar Menú").clicked() {
                        self.show_copy_confirm_dialog = true;
                    }
                    if ui.button("💾 Exportar").clicked() {
                        trigger_export = true;
                    }
                    if ui.button("⚙ Objectius").clicked() {
                        self.show_edit_goals_modal = true;
                    }
                    if ui.button("🗑 Buidar").clicked() {
                        for entries in state.daily_journal[log_idx].daily_menu.meals.values_mut() {
                            entries.clear();
                        }
                    }
                });
            } else {
                // Desktop layout for Date Navigation
                ui.horizontal(|ui| {
                    if ui.button("◀ Dia Anterior").clicked() {
                        self.selected_date = offset_date(&self.selected_date, -1);
                    }
                    if ui.selectable_label(is_today, "📅 Avui").clicked() {
                        self.selected_date = today_str.clone();
                    }
                    if ui.button("Dia Següent ▶").clicked() {
                        self.selected_date = offset_date(&self.selected_date, 1);
                    }

                    ui.separator();
                    ui.heading(format!("📅 {}", formatted_date));
                    if is_today {
                        ui.colored_label(Color32::from_rgb(34, 197, 94), " [AVUI]");
                    }

                    // Jump to existing recorded days combo
                    if state.daily_journal.len() > 1 {
                        ui.separator();
                        ui.label("Historial:");
                        let current_d = self.selected_date.clone();
                        egui::ComboBox::from_id_salt("journal_date_history_combo")
                            .selected_text(&current_d)
                            .show_ui(ui, |ui| {
                                for log in &state.daily_journal {
                                    let is_selected = self.selected_date == log.date;
                                    let text = format!("{} ({})", log.date, log.daily_menu.day_name);
                                    if ui.selectable_label(is_selected, text).clicked() {
                                        self.selected_date = log.date.clone();
                                    }
                                }
                            });
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑 Buidar Dia").clicked() {
                            for entries in state.daily_journal[log_idx].daily_menu.meals.values_mut() {
                                entries.clear();
                            }
                        }
                        if ui.button("⚙ Personalitzar Objectius").clicked() {
                            self.show_edit_goals_modal = true;
                        }
                        if ui.button("💾 Guardar Registre en PDF / HTML...").clicked() {
                            trigger_export = true;
                        }
                        if ui.button("📋 Copiar del Menú Planificat").clicked() {
                            self.show_copy_confirm_dialog = true;
                        }
                    });
                });
            }
        });

        if trigger_export {
            self.export_daily_log(state);
        }

        if let Some(msg) = &self.export_message {
            ui.add_space(4.0);
            ui.colored_label(Color32::from_rgb(34, 197, 94), msg);
        }

        ui.add_space(6.0);

        // Destructure state to avoid borrowing conflicts
        let all_ingredients = state.all_ingredients();
        let AppState { ingredients, dishes, weekly_menu, goals, daily_journal, punctual_ingredients } = state;
        let log = &mut daily_journal[log_idx];
        let total_nut = log.daily_menu.calculate_total_nutrition(&all_ingredients, dishes);

        // SECTION 1: Top Calorie & Macro Hero Card (Clean Nutrition Dashboard)
        ui.group(|ui| {
            let remaining_kcal = goals.max_kcal - total_nut.kcal;
            let kcal_ratio = (total_nut.kcal / goals.max_kcal).clamp(0.0, 1.0);

            let (status_text, kcal_color) = if total_nut.kcal < goals.min_kcal {
                (format!("⚡ Resten {:.0} kcal per l'objectiu", remaining_kcal.max(0.0)), Color32::from_rgb(251, 146, 60))
            } else if total_nut.kcal <= goals.max_kcal {
                (format!("✅ Dins del rang ({:.0} kcal restants)", remaining_kcal.max(0.0)), Color32::from_rgb(34, 197, 94))
            } else {
                (format!("⚠ Superat per {:.0} kcal", total_nut.kcal - goals.max_kcal), Color32::from_rgb(239, 68, 68))
            };

            ui.horizontal_wrapped(|ui| {
                ui.heading(format!("⚡ {:.0} / {:.0} Kcal", total_nut.kcal, goals.max_kcal));
                ui.colored_label(kcal_color, status_text);
            });

            ui.add_space(2.0);
            ui.add(ProgressBar::new(kcal_ratio as f32).fill(kcal_color));
            ui.add_space(6.0);

            // Macro Bars Card
            let prot_ratio = if goals.min_protein_g > 0.0 { (total_nut.protein_g / goals.min_protein_g).clamp(0.0, 1.0) } else { 0.0 };
            let carb_ratio = if goals.max_carbs_pct > 0.0 { (total_nut.carbs_pct() / goals.max_carbs_pct).clamp(0.0, 1.0) } else { 0.0 };
            let fat_ratio = if goals.max_fat_pct > 0.0 { (total_nut.fat_pct() / goals.max_fat_pct).clamp(0.0, 1.0) } else { 0.0 };

            if is_mobile {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(Color32::from_rgb(248, 113, 113), format!("🥩 P: {:.1}g ({:.0}%)", total_nut.protein_g, total_nut.protein_pct()));
                    ui.colored_label(Color32::from_rgb(56, 189, 248), format!("🌾 C: {:.1}g ({:.0}%)", total_nut.carbs_g, total_nut.carbs_pct()));
                    ui.colored_label(Color32::from_rgb(234, 179, 8), format!("🥑 G: {:.1}g ({:.0}%)", total_nut.fat_g, total_nut.fat_pct()));
                    ui.colored_label(Color32::from_rgb(52, 211, 153), format!("🌿 Fibra: {:.1}g", total_nut.fiber_g));
                    ui.colored_label(Color32::from_rgb(167, 139, 250), format!("💶 {:.2} €", total_nut.price_euro));
                });
            } else {
                ui.columns(3, |cols| {
                    cols[0].group(|ui| {
                        ui.colored_label(Color32::from_rgb(248, 113, 113), format!("🥩 Proteïna: {:.1}g ({:.0}%)", total_nut.protein_g, total_nut.protein_pct()));
                        ui.add(ProgressBar::new(prot_ratio as f32).fill(Color32::from_rgb(248, 113, 113)));
                    });
                    cols[1].group(|ui| {
                        ui.colored_label(Color32::from_rgb(56, 189, 248), format!("🌾 Hidrats: {:.1}g ({:.0}%)", total_nut.carbs_g, total_nut.carbs_pct()));
                        ui.add(ProgressBar::new(carb_ratio as f32).fill(Color32::from_rgb(56, 189, 248)));
                    });
                    cols[2].group(|ui| {
                        ui.colored_label(Color32::from_rgb(234, 179, 8), format!("🥑 Greixos: {:.1}g ({:.0}%)", total_nut.fat_g, total_nut.fat_pct()));
                        ui.add(ProgressBar::new(fat_ratio as f32).fill(Color32::from_rgb(234, 179, 8)));
                    });
                });
            }
        });

        ui.add_space(8.0);

        // SECTION 2: Meals of the Selected Day
        for meal_type in MealType::all() {
            let entries = log.daily_menu.meals.entry(meal_type).or_insert_with(Vec::new);
            let mut meal_kcal = 0.0;
            let mut meal_price = 0.0;
            for entry in entries.iter() {
                if entry.is_dish {
                    if let Some(dish) = dishes.iter().find(|d| d.id == entry.item_id) {
                        let nut = dish.calculate_total_nutrition(&all_ingredients).scale(entry.quantity);
                        meal_kcal += nut.kcal;
                        meal_price += nut.price_euro;
                    }
                } else if let Some(ing) = all_ingredients.iter().find(|i| i.id == entry.item_id) {
                    let nut = ing.calculate_nutrition(entry.quantity);
                    meal_kcal += nut.kcal;
                    meal_price += nut.price_euro;
                }
            }

            ui.group(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.heading(meal_type.name_ca());
                    if meal_kcal > 0.0 {
                        ui.colored_label(Color32::from_rgb(251, 146, 60), format!("({:.0} kcal • {:.2} €)", meal_kcal, meal_price));
                    }
                    if ui.button("➕ Afegir").clicked() {
                        self.selected_meal = Some(meal_type);
                        self.show_add_item_dialog = true;
                        self.add_item_id.clear();
                        self.picker_search.clear();
                        self.add_quantity = 100.0;
                    }
                });

                ui.separator();

                if entries.is_empty() {
                    ui.label(" (Sense aliments registrats per a aquest àpat)");
                } else if is_mobile {
                    // Mobile Card Layout
                    let mut to_remove = None;
                    for (entry_idx, entry) in entries.iter_mut().enumerate() {
                        ui.group(|ui| {
                            if entry.is_dish {
                                if let Some(dish) = dishes.iter().find(|d| d.id == entry.item_id) {
                                    let nova = dish.derived_nova_group(&all_ingredients);
                                    let ig = dish.derived_glycemic_index(&all_ingredients);
                                    let level = dish.derived_glycemic_level(&all_ingredients);
                                    let cg = dish.calculate_glycemic_load(&all_ingredients) * entry.quantity;
                                    let nut = dish.calculate_total_nutrition(&all_ingredients).scale(entry.quantity);

                                    ui.horizontal_wrapped(|ui| {
                                        draw_nova_badge(ui, nova);
                                        draw_glycemic_badge(ui, level, ig);
                                        ui.strong(format!("🍲 {}", dish.name));
                                        if ui.small_button("🗑").clicked() {
                                            to_remove = Some(entry_idx);
                                        }
                                    });

                                    ui.horizontal_wrapped(|ui| {
                                        ui.label("Racions:");
                                        ui.add(egui::DragValue::new(&mut entry.quantity).speed(0.1).range(0.1..=20.0).max_decimals(2));
                                        ui.colored_label(Color32::from_rgb(251, 146, 60), format!("⚡ {:.0} kcal", nut.kcal));
                                        ui.colored_label(Color32::from_rgb(52, 211, 153), format!("🏷 {:.2} €", nut.price_euro));
                                    });

                                    ui.horizontal_wrapped(|ui| {
                                        ui.small(format!("🥩 P: {:.1}g", nut.protein_g));
                                        ui.small(format!("🌾 C: {:.1}g (CG {:.1})", nut.carbs_g, cg));
                                        ui.small(format!("🥑 G: {:.1}g", nut.fat_g));
                                        ui.small(format!("🌿 Fibra: {:.1}g", nut.fiber_g));
                                    });
                                } else {
                                    ui.label(format!("Plat desconegut ({})", entry.item_id));
                                }
                            } else {
                                if let Some(ing) = all_ingredients.iter().find(|i| i.id == entry.item_id) {
                                    let is_punctual = punctual_ingredients.iter().any(|p| p.id == ing.id);
                                    let nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                                    let ig = ing.get_glycemic_index();
                                    let level = ing.get_glycemic_level();
                                    let cg = ing.calculate_glycemic_load(entry.quantity);
                                    let nut = ing.calculate_nutrition(entry.quantity);
                                    let (speed, unit_str, max_range) = match ing.unit_type {
                                        UnitType::Per100g => (0.1, "g", 3000.0),
                                        UnitType::PerUnit { .. } => (0.1, "ut", 50.0),
                                    };

                                    ui.horizontal_wrapped(|ui| {
                                        draw_nova_badge(ui, nova);
                                        draw_glycemic_badge(ui, level, ig);
                                        let name_str = if is_punctual {
                                            format!("✨ {} [Puntual]", ing.name)
                                        } else {
                                            format!("🥗 {}", ing.name)
                                        };
                                        ui.strong(name_str);
                                        if ui.small_button("🗑").clicked() {
                                            to_remove = Some(entry_idx);
                                        }
                                    });

                                    ui.horizontal_wrapped(|ui| {
                                        ui.label("Quantitat:");
                                        ui.add(egui::DragValue::new(&mut entry.quantity).speed(speed).range(0.01..=max_range).max_decimals(2));
                                        ui.small(unit_str);
                                        ui.colored_label(Color32::from_rgb(251, 146, 60), format!("⚡ {:.0} kcal", nut.kcal));
                                        ui.colored_label(Color32::from_rgb(52, 211, 153), format!("🏷 {:.2} €", nut.price_euro));
                                    });

                                    ui.horizontal_wrapped(|ui| {
                                        ui.small(format!("🥩 P: {:.1}g", nut.protein_g));
                                        ui.small(format!("🌾 C: {:.1}g (CG {:.1})", nut.carbs_g, cg));
                                        ui.small(format!("🥑 G: {:.1}g", nut.fat_g));
                                        ui.small(format!("🌿 Fibra: {:.1}g", nut.fiber_g));
                                    });
                                } else {
                                    ui.label(format!("Aliment desconegut ({})", entry.item_id));
                                }
                            }
                        });
                        ui.add_space(2.0);
                    }

                    if let Some(rem_idx) = to_remove {
                        entries.remove(rem_idx);
                    }
                } else {
                    // Desktop table layout
                    egui::Grid::new(format!("journal_grid_{:?}", meal_type))
                        .striped(true)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.strong("Element");
                            ui.strong("Quantitat");
                            ui.strong("Kcal");
                            ui.strong("Greixos");
                            ui.strong("HdC (Sucres)");
                            ui.strong("Fibra");
                            ui.strong("Proteïna");
                            ui.strong("Sal");
                            ui.strong("Preu (€)");
                            ui.strong("Acció");
                            ui.end_row();

                            let mut to_remove = None;

                            for (entry_idx, entry) in entries.iter_mut().enumerate() {
                                if entry.is_dish {
                                    if let Some(dish) = dishes.iter().find(|d| d.id == entry.item_id) {
                                        let nova = dish.derived_nova_group(&all_ingredients);
                                        let ig = dish.derived_glycemic_index(&all_ingredients);
                                        let level = dish.derived_glycemic_level(&all_ingredients);
                                        let cg = dish.calculate_glycemic_load(&all_ingredients) * entry.quantity;

                                        ui.horizontal(|ui| {
                                            draw_nova_badge(ui, nova);
                                            draw_glycemic_badge(ui, level, ig);
                                            ui.label(format!("🍲 {}", dish.name));
                                        });

                                        ui.horizontal(|ui| {
                                            ui.add(egui::DragValue::new(&mut entry.quantity).speed(0.1).range(0.1..=20.0).max_decimals(2));
                                            ui.small("racions");
                                        });

                                        let nut = dish.calculate_total_nutrition(&all_ingredients).scale(entry.quantity);
                                        ui.label(format!("{:.0}", nut.kcal));
                                        ui.label(format!("{:.1}g ({:.1}g)", nut.fat_g, nut.saturated_fat_g));
                                        ui.label(format!("{:.1}g ({:.1}g) [CG {:.1}]", nut.carbs_g, nut.sugars_g, cg));
                                        ui.label(format!("{:.1}g", nut.fiber_g));
                                        ui.label(format!("{:.1}g", nut.protein_g));
                                        ui.label(format!("{:.2}g", nut.salt_g));
                                        ui.label(format!("{:.2}€", nut.price_euro));
                                    } else {
                                        ui.label(format!("Plat desconegut ({})", entry.item_id));
                                        for _ in 0..8 { ui.label("-"); }
                                    }
                                } else {
                                    if let Some(ing) = all_ingredients.iter().find(|i| i.id == entry.item_id) {
                                        let is_punctual = punctual_ingredients.iter().any(|p| p.id == ing.id);
                                        let nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                                        let ig = ing.get_glycemic_index();
                                        let level = ing.get_glycemic_level();
                                        let cg = ing.calculate_glycemic_load(entry.quantity);

                                        ui.horizontal(|ui| {
                                            draw_nova_badge(ui, nova);
                                            draw_glycemic_badge(ui, level, ig);
                                            let name_str = if is_punctual {
                                                format!("✨ {} [Puntual]", ing.name)
                                            } else {
                                                format!("🥗 {}", ing.name)
                                            };
                                            ui.label(name_str);
                                        });

                                        let (speed, unit_str, max_range) = match ing.unit_type {
                                            UnitType::Per100g => (0.1, "g", 3000.0),
                                            UnitType::PerUnit { .. } => (0.1, "ut", 50.0),
                                        };
                                        ui.horizontal(|ui| {
                                            ui.add(egui::DragValue::new(&mut entry.quantity).speed(speed).range(0.01..=max_range).max_decimals(2));
                                            ui.small(unit_str);
                                        });

                                        let nut = ing.calculate_nutrition(entry.quantity);
                                        ui.label(format!("{:.0}", nut.kcal));
                                        ui.label(format!("{:.1}g ({:.1}g)", nut.fat_g, nut.saturated_fat_g));
                                        ui.label(format!("{:.1}g ({:.1}g) [CG {:.1}]", nut.carbs_g, nut.sugars_g, cg));
                                        ui.label(format!("{:.1}g", nut.fiber_g));
                                        ui.label(format!("{:.1}g", nut.protein_g));
                                        ui.label(format!("{:.2}g", nut.salt_g));
                                        ui.label(format!("{:.2}€", nut.price_euro));
                                    } else {
                                        ui.label(format!("Aliment desconegut ({})", entry.item_id));
                                        for _ in 0..8 { ui.label("-"); }
                                    }
                                }

                                if ui.small_button("🗑").clicked() {
                                    to_remove = Some(entry_idx);
                                }

                                ui.end_row();
                            }

                            if let Some(rem_idx) = to_remove {
                                entries.remove(rem_idx);
                            }
                        });
                }
            });
            ui.add_space(6.0);
        }

        ui.add_space(8.0);

        // SECTION 3: Detailed Nutritional Dashboard & Goals Assessment
        ui.group(|ui| {
            ui.heading("📊 Resum Nutricional Detallat i Objectius");
            ui.add_space(6.0);

            let day_menu = &log.daily_menu;

            ui.group(|ui| {
                ui.strong("📈 Índex i Càrrega Glucèmica (CG)");
                let total_cg = day_menu.calculate_total_glycemic_load(&all_ingredients, dishes);
                let cg_ratio = (total_cg / goals.max_glycemic_load).clamp(0.0, 1.0);
                let cg_color = if total_cg <= goals.max_glycemic_load {
                    Color32::from_rgb(34, 197, 94)
                } else {
                    Color32::from_rgb(239, 68, 68)
                };

                ui.add(ProgressBar::new(cg_ratio as f32).fill(cg_color).text(format!("CG Diària: {:.1} / {:.0}", total_cg, goals.max_glycemic_load)));
                if total_cg <= goals.max_glycemic_load {
                    ui.colored_label(Color32::from_rgb(34, 197, 94), "✅ Càrrega Glucèmica sota el límit recomanat");
                } else {
                    ui.colored_label(Color32::from_rgb(239, 68, 68), format!("⚠ Càrrega Glucèmica elevada ({:.1}). Redueix refinats i sucres.", total_cg));
                }
            });

            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("🏷 Classificació NOVA");
                let nova_map = day_menu.nova_breakdown(&all_ingredients, dishes);
                let total_k = total_nut.kcal;

                for group in [NovaGroup::Group1Unprocessed, NovaGroup::Group2ProcessedIngredient, NovaGroup::Group3Processed, NovaGroup::Group4UltraProcessed] {
                    let kcal_g = nova_map.get(&group).copied().unwrap_or(0.0);
                    let pct = if total_k > 0.0 { (kcal_g / total_k) * 100.0 } else { 0.0 };

                    ui.horizontal_wrapped(|ui| {
                        draw_nova_badge(ui, group);
                        ui.label(format!("{:.1}% ({:.0} Kcal)", pct, kcal_g));
                    });
                }

                let ultra_kcal = nova_map.get(&NovaGroup::Group4UltraProcessed).copied().unwrap_or(0.0);
                let ultra_pct = if total_k > 0.0 { (ultra_kcal / total_k) * 100.0 } else { 0.0 };
                if ultra_pct > goals.max_ultraprocessed_pct {
                    ui.colored_label(Color32::from_rgb(239, 68, 68), format!("⚠ Alt contingut d'Ultraprocessats ({:.1}% de Kcal)", ultra_pct));
                } else {
                    ui.colored_label(Color32::from_rgb(34, 197, 94), "✅ Dieta neta d'ultraprocessats");
                }
            });

            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("⚠ Límits i Control de Nutrients");
                // Sat Fat
                let sat_ok = total_nut.saturated_fat_g <= goals.max_saturated_fat_g;
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("• Greixos Saturats: {:.1}g (Màx: {:.0}g)", total_nut.saturated_fat_g, goals.max_saturated_fat_g));
                    ui.colored_label(if sat_ok { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(239, 68, 68) }, if sat_ok { "OK" } else { "EXCEDIT" });
                });

                // Sugars
                let sugar_p = total_nut.sugar_pct_of_macros();
                let sugar_ok = sugar_p <= goals.max_sugar_macro_pct;
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("• Sucres: {:.1}% de macros (Màx: {:.0}%)", sugar_p, goals.max_sugar_macro_pct));
                    ui.colored_label(if sugar_ok { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(239, 68, 68) }, if sugar_ok { "OK" } else { "EXCEDIT" });
                });

                // Fiber
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("• Fibra: {:.1}g (Mín: {:.0}g | Ideal: {:.0}g)", total_nut.fiber_g, goals.min_fiber_g, goals.ideal_fiber_g));
                    if total_nut.fiber_g >= goals.ideal_fiber_g {
                        ui.colored_label(Color32::from_rgb(34, 197, 94), "Excel·lent");
                    } else if total_nut.fiber_g >= goals.min_fiber_g {
                        ui.colored_label(Color32::from_rgb(34, 197, 94), "Acceptable");
                    } else {
                        ui.colored_label(Color32::from_rgb(251, 146, 60), "Insuficient");
                    }
                });

                // Salt
                let salt_ok = total_nut.salt_g <= goals.max_salt_g;
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("• Sal: {:.2}g (Màx: {:.0}g)", total_nut.salt_g, goals.max_salt_g));
                    ui.colored_label(if salt_ok { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(239, 68, 68) }, if salt_ok { "OK" } else { "EXCEDIT" });
                });
            });

            ui.add_space(4.0);
            ui.group(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.strong("💶 Despesa Total del Dia:");
                    ui.heading(format!("{:.2} €", total_nut.price_euro));
                });
            });
        });

        // Copy Plan Confirmation Dialog
        if self.show_copy_confirm_dialog {
            let mut close_dialog = false;
            let mut do_copy = false;

            egui::Window::new("📋 Copiar Menú Planificat")
                .collapsible(false)
                .resizable(false)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(screen_rect.center())
                .max_width(modal_width)
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "Selecciona el menú que vols copiar al registre del dia {}:",
                        self.selected_date
                    ));
                    ui.add_space(6.0);

                    if self.copy_source_day_idx >= weekly_menu.days.len() {
                        self.copy_source_day_idx = 0;
                    }

                    let current_source_name = weekly_menu.days.get(self.copy_source_day_idx)
                        .map(|d| d.day_name.as_str())
                        .unwrap_or("Menú");

                    egui::ComboBox::from_id_salt("journal_copy_source_combo")
                        .selected_text(format!("📋 {}", current_source_name))
                        .show_ui(ui, |ui| {
                            for (idx, day) in weekly_menu.days.iter().enumerate() {
                                let is_selected = self.copy_source_day_idx == idx;
                                if ui.selectable_label(is_selected, format!("📋 {}", day.day_name)).clicked() {
                                    self.copy_source_day_idx = idx;
                                }
                            }
                        });

                    ui.add_space(8.0);
                    ui.colored_label(Color32::from_rgb(251, 146, 60), "Nota: Això substituirà els àpats actuals d'aquesta data.");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("✅ Sí, copiar").clicked() {
                            do_copy = true;
                            close_dialog = true;
                        }
                        if ui.button("Cancel·lar").clicked() {
                            close_dialog = true;
                        }
                    });
                });

            if do_copy {
                if let Some(planned_day) = weekly_menu.days.get(self.copy_source_day_idx) {
                    log.daily_menu.meals = planned_day.meals.clone();
                    self.export_message = Some(format!("📋 S'han copiat els àpats de '{}' al registre diari!", planned_day.day_name));
                }
            }

            if close_dialog {
                self.show_copy_confirm_dialog = false;
            }
        }

        // Add Item Modal Dialog
        if self.show_add_item_dialog {
            let mut close_modal = false;
            let meal = self.selected_meal.unwrap_or(MealType::Breakfast);

            let modal_width_dyn = if is_mobile { (screen_rect.width() - 20.0).max(280.0) } else { 620.0 };
            let modal_height_dyn = if is_mobile { (screen_rect.height() - 40.0).max(350.0) } else { 580.0 };

            egui::Window::new(format!("Afegir Element a: {}", meal.name_ca()))
                .collapsible(false)
                .resizable(true)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(screen_rect.center())
                .default_width(modal_width_dyn)
                .max_width(modal_width_dyn)
                .max_height(modal_height_dyn)
                .show(ui.ctx(), |ui| {
                    ui.horizontal_wrapped(|ui| {
                        if ui.selectable_label(self.add_category == AddItemCategory::Ingredient, "🥗 Aliments").clicked() {
                            self.add_category = AddItemCategory::Ingredient;
                            self.add_is_dish = false;
                            self.add_item_id.clear();
                        }
                        if ui.selectable_label(self.add_category == AddItemCategory::Dish, "🍲 Plats").clicked() {
                            self.add_category = AddItemCategory::Dish;
                            self.add_is_dish = true;
                            self.add_item_id.clear();
                        }
                        if ui.selectable_label(self.add_category == AddItemCategory::Punctual, "✨ Aliment Puntual").clicked() {
                            self.add_category = AddItemCategory::Punctual;
                            self.add_is_dish = false;
                            self.add_item_id.clear();
                        }
                    });

                    ui.separator();

                    if self.add_category == AddItemCategory::Punctual {
                        egui::ScrollArea::vertical().max_height(modal_height_dyn - 90.0).show(ui, |ui| {
                            ui.label("💡 Registra un aliment puntual (ex: àpat de restaurant, estimació per foto d'IA) només per al seguiment d'avui, sense afegir-lo a la base de dades general d'aliments.");
                            ui.add_space(4.0);

                            // 1. JSON Import section
                            ui.group(|ui| {
                                ui.strong("🤖 Importar des de JSON (Agent IA / Chatbot)");
                                ui.label("Pots enganxar el text JSON de l'aliment estimat per la teva IA:");
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.punctual_json_paste)
                                        .desired_rows(4)
                                        .desired_width(f32::INFINITY)
                                        .hint_text(r#"{"name": "...", "per_unit_nutrition": { "kcal": 450, ... }, "nova_group": "Group3Processed"}"#)
                                );
                                ui.horizontal_wrapped(|ui| {
                                    if ui.button("📥 Omplir Formulari des del JSON").clicked() {
                                        self.load_punctual_from_json();
                                    }
                                    if ui.button("🧹 Netejar JSON").clicked() {
                                        self.punctual_json_paste.clear();
                                        self.punctual_json_status = None;
                                    }
                                });
                                if let Some(status) = &self.punctual_json_status {
                                    if status.starts_with("✅") {
                                        ui.colored_label(Color32::from_rgb(34, 197, 94), status);
                                    } else {
                                        ui.colored_label(Color32::from_rgb(239, 68, 68), status);
                                    }
                                }
                            });

                            ui.add_space(4.0);

                            // 2. Manual Food Form
                            ui.group(|ui| {
                                ui.strong("📝 Formulari de l'Aliment");
                                ui.horizontal_wrapped(|ui| {
                                    ui.label("Nom:");
                                    ui.text_edit_singleline(&mut self.punctual_name);
                                });
                                ui.horizontal_wrapped(|ui| {
                                    ui.small("Accents:");
                                    for c in ["à", "è", "é", "í", "ï", "ò", "ó", "ú", "ü", "ç", "·"] {
                                        if ui.small_button(c).clicked() {
                                            self.punctual_name.push_str(c);
                                        }
                                    }
                                });
                                ui.horizontal_wrapped(|ui| {
                                    ui.label("Lloc / Marca:");
                                    ui.text_edit_singleline(&mut self.punctual_brand);
                                });

                                ui.separator();
                                ui.label("Valors Nutricionals (per 100g):");
                                ui.columns(2, |cols| {
                                    cols[0].horizontal_wrapped(|ui| { ui.label("Kcal:"); ui.add(egui::DragValue::new(&mut self.punctual_kcal).speed(1.0).range(0.0..=900.0)); });
                                    cols[0].horizontal_wrapped(|ui| { ui.label("Greixos (g):"); ui.add(egui::DragValue::new(&mut self.punctual_fat).speed(0.1).range(0.0..=100.0)); });
                                    cols[0].horizontal_wrapped(|ui| { ui.label("Greixos Sat (g):"); ui.add(egui::DragValue::new(&mut self.punctual_sat_fat).speed(0.1).range(0.0..=100.0)); });
                                    cols[0].horizontal_wrapped(|ui| { ui.label("HdC (g):"); ui.add(egui::DragValue::new(&mut self.punctual_carbs).speed(0.1).range(0.0..=100.0)); });

                                    cols[1].horizontal_wrapped(|ui| { ui.label("Sucres (g):"); ui.add(egui::DragValue::new(&mut self.punctual_sugar).speed(0.1).range(0.0..=100.0)); });
                                    cols[1].horizontal_wrapped(|ui| { ui.label("Fibra (g):"); ui.add(egui::DragValue::new(&mut self.punctual_fiber).speed(0.1).range(0.0..=100.0)); });
                                    cols[1].horizontal_wrapped(|ui| { ui.label("Proteïna (g):"); ui.add(egui::DragValue::new(&mut self.punctual_protein).speed(0.1).range(0.0..=100.0)); });
                                    cols[1].horizontal_wrapped(|ui| { ui.label("Sal (g):"); ui.add(egui::DragValue::new(&mut self.punctual_salt).speed(0.01).range(0.0..=100.0)); });
                                });

                                ui.separator();
                                ui.horizontal_wrapped(|ui| {
                                    ui.label("Preu per 100g / ració (€):");
                                    ui.add(egui::DragValue::new(&mut self.punctual_price).speed(0.05).range(0.0..=500.0));
                                });

                                ui.horizontal_wrapped(|ui| {
                                    ui.label("Classificació NOVA:");
                                    egui::ComboBox::from_id_salt("punctual_nova_combo")
                                        .selected_text(self.punctual_nova.short_name_ca())
                                        .show_ui(ui, |ui| {
                                            for group in [NovaGroup::Group1Unprocessed, NovaGroup::Group2ProcessedIngredient, NovaGroup::Group3Processed, NovaGroup::Group4UltraProcessed] {
                                                ui.selectable_value(&mut self.punctual_nova, group, group.name_ca());
                                            }
                                        });
                                });

                                ui.horizontal_wrapped(|ui| {
                                    ui.label("Índex Glucèmic (IG):");
                                    ui.add(egui::Slider::new(&mut self.punctual_ig, 0..=100).text("IG"));
                                    let level = GlycemicLevel::from_ig(self.punctual_ig);
                                    draw_glycemic_badge(ui, level, self.punctual_ig);
                                });

                                ui.horizontal_wrapped(|ui| {
                                    ui.label("Ingredients (opcional):");
                                    ui.text_edit_singleline(&mut self.punctual_ingredients_text);
                                });
                            });

                            ui.add_space(4.0);

                            // 3. Quantity to add
                            ui.group(|ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.strong("Quantitat a consumir en aquest àpat (g):");
                                    ui.add(egui::DragValue::new(&mut self.add_quantity).speed(1.0).range(0.1..=3000.0).max_decimals(1));
                                });
                            });

                            // 4. Action buttons
                            ui.separator();
                            ui.horizontal_wrapped(|ui| {
                                if ui.button("➕ Afegir Aliment Puntual a l'Àpat").clicked() {
                                    if !self.punctual_name.trim().is_empty() {
                                        let mut i = 1;
                                        let punctual_id = loop {
                                            let candidate = format!("punctual_{}", i);
                                            if !punctual_ingredients.iter().any(|ing| ing.id == candidate)
                                                && !ingredients.iter().any(|ing| ing.id == candidate) {
                                                break candidate;
                                            }
                                            i += 1;
                                        };

                                        let brand_opt = if self.punctual_brand.trim().is_empty() { None } else { Some(self.punctual_brand.trim().to_string()) };
                                        let ing_text_opt = if self.punctual_ingredients_text.trim().is_empty() { None } else { Some(self.punctual_ingredients_text.trim().to_string()) };

                                        let new_punctual_ing = Ingredient {
                                            id: punctual_id.clone(),
                                            name: self.punctual_name.trim().to_string(),
                                            brand: brand_opt,
                                            source_url: None,
                                            unit_type: UnitType::Per100g,
                                            per_unit_nutrition: NutritionalInfo {
                                                kcal: self.punctual_kcal,
                                                fat_g: self.punctual_fat,
                                                saturated_fat_g: self.punctual_sat_fat,
                                                carbs_g: self.punctual_carbs,
                                                sugars_g: self.punctual_sugar,
                                                fiber_g: self.punctual_fiber,
                                                protein_g: self.punctual_protein,
                                                salt_g: self.punctual_salt,
                                                price_euro: self.punctual_price,
                                            },
                                            price_per_pack: None,
                                            pack_weight_g: None,
                                            nova_group: Some(self.punctual_nova),
                                            glycemic_index: Some(self.punctual_ig),
                                            ingredients_text: ing_text_opt,
                                        };

                                        punctual_ingredients.push(new_punctual_ing);

                                        let entries = log.daily_menu.meals.entry(meal).or_insert_with(Vec::new);
                                        entries.push(MealEntry {
                                            id: format!("{}_{}", punctual_id, entries.len()),
                                            item_id: punctual_id,
                                            is_dish: false,
                                            quantity: self.add_quantity,
                                        });

                                        self.reset_punctual_form();
                                        close_modal = true;
                                    } else {
                                        self.punctual_json_status = Some("⚠️ Introdueix un nom per a l'aliment puntual abans d'afegir-lo.".to_string());
                                    }
                                }
                                if ui.button("Cancel·lar").clicked() {
                                    close_modal = true;
                                }
                            });

                            // 5. Previous punctual foods
                            if !punctual_ingredients.is_empty() {
                                ui.add_space(8.0);
                                ui.separator();
                                ui.collapsing(format!("🕒 Aliments Puntuals Anteriors ({})", punctual_ingredients.len()), |ui| {
                                    ui.small("Fes clic a '📋 Copiar' per carregar les dades al formulari:");
                                    for p_ing in punctual_ingredients.iter().rev() {
                                        ui.horizontal_wrapped(|ui| {
                                            let nova = p_ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                                            draw_nova_badge(ui, nova);
                                            ui.label(format!("✨ {} ({:.0} kcal)", p_ing.name, p_ing.per_unit_nutrition.kcal));
                                            if ui.small_button("📋 Copiar").clicked() {
                                                self.punctual_name = p_ing.name.clone();
                                                self.punctual_brand = p_ing.brand.clone().unwrap_or_default();
                                                self.punctual_kcal = p_ing.per_unit_nutrition.kcal;
                                                self.punctual_fat = p_ing.per_unit_nutrition.fat_g;
                                                self.punctual_sat_fat = p_ing.per_unit_nutrition.saturated_fat_g;
                                                self.punctual_carbs = p_ing.per_unit_nutrition.carbs_g;
                                                self.punctual_sugar = p_ing.per_unit_nutrition.sugars_g;
                                                self.punctual_fiber = p_ing.per_unit_nutrition.fiber_g;
                                                self.punctual_protein = p_ing.per_unit_nutrition.protein_g;
                                                self.punctual_salt = p_ing.per_unit_nutrition.salt_g;
                                                self.punctual_price = p_ing.per_unit_nutrition.price_euro;
                                                self.punctual_nova = p_ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                                                self.punctual_ig = p_ing.get_glycemic_index();
                                                self.punctual_ingredients_text = p_ing.ingredients_text.clone().unwrap_or_default();
                                            }
                                        });
                                    }
                                });
                            }
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("🔍 Cercar:");
                            ui.text_edit_singleline(&mut self.picker_search);
                        });

                        ui.add_space(4.0);

                        // Scrollable list of items
                        let scroll_max_h = if is_mobile { 180.0 } else { 240.0 };
                        egui::ScrollArea::vertical().max_height(scroll_max_h).show(ui, |ui| {
                            let q = self.picker_search.to_lowercase();
                            if self.add_is_dish {
                                if dishes.is_empty() {
                                    ui.label("Cap plat creat encara. Ves a 'Plats' per crear-ne.");
                                } else {
                                    for dish in dishes.iter() {
                                        if !q.is_empty() && !matches_search(&dish.name, &q) {
                                            continue;
                                        }
                                        let is_sel = self.add_item_id == dish.id;
                                        ui.horizontal_wrapped(|ui| {
                                            let nova = dish.derived_nova_group(&all_ingredients);
                                            let ig = dish.derived_glycemic_index(&all_ingredients);
                                            let level = dish.derived_glycemic_level(&all_ingredients);
                                            draw_nova_badge(ui, nova);
                                            draw_glycemic_badge(ui, level, ig);
                                            let nut = dish.calculate_total_nutrition(&all_ingredients);
                                            let label_text = format!("🍲 {} ({:.0} Kcal)", dish.name, nut.kcal);

                                            if ui.selectable_label(is_sel, label_text).clicked() {
                                                self.add_item_id = dish.id.clone();
                                                self.add_quantity = 1.0;
                                            }
                                        });
                                    }
                                }
                            } else {
                                if ingredients.is_empty() {
                                    ui.label("La base de dades d'aliments està buida.");
                                } else {
                                    for ing in ingredients.iter() {
                                        if !q.is_empty() 
                                            && !matches_search(&ing.name, &q) 
                                            && !matches_search(ing.brand.as_deref().unwrap_or(""), &q) 
                                        {
                                            continue;
                                        }
                                        let is_sel = self.add_item_id == ing.id;
                                        ui.horizontal_wrapped(|ui| {
                                            let nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                                            let ig = ing.get_glycemic_index();
                                            let level = ing.get_glycemic_level();
                                            draw_nova_badge(ui, nova);
                                            draw_glycemic_badge(ui, level, ig);
                                            let name_display = if let Some(b) = &ing.brand {
                                                format!("{} [{}] ({:.0} Kcal)", ing.name, b, ing.per_unit_nutrition.kcal)
                                            } else {
                                                format!("{} ({:.0} Kcal)", ing.name, ing.per_unit_nutrition.kcal)
                                            };

                                            if ui.selectable_label(is_sel, name_display).clicked() {
                                                self.add_item_id = ing.id.clone();
                                                self.add_quantity = match ing.unit_type {
                                                    UnitType::Per100g => 100.0,
                                                    UnitType::PerUnit { .. } => 1.0,
                                                };
                                            }
                                        });
                                    }
                                }
                            }
                        });

                        ui.separator();

                        if !self.add_item_id.is_empty() {
                            if self.add_is_dish {
                                if let Some(dish) = dishes.iter().find(|d| d.id == self.add_item_id) {
                                    ui.strong(format!("Seleccionat: 🍲 {}", dish.name));
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label("Racions:");
                                        ui.add(egui::DragValue::new(&mut self.add_quantity).speed(0.1).range(0.1..=20.0));
                                    });
                                }
                            } else {
                                if let Some(ing) = ingredients.iter().find(|i| i.id == self.add_item_id) {
                                    ui.strong(format!("Seleccionat: 🥗 {}", ing.name));
                                    let (label_text, speed, max_val) = match ing.unit_type {
                                        UnitType::Per100g => ("Quantitat (g):", 0.1, 3000.0),
                                        UnitType::PerUnit { .. } => ("Unitats:", 0.1, 50.0),
                                    };
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(label_text);
                                        ui.add(egui::DragValue::new(&mut self.add_quantity).speed(speed).range(0.01..=max_val).max_decimals(2));
                                    });
                                }
                            }
                        } else {
                            ui.colored_label(Color32::from_rgb(251, 146, 60), "👈 Selecciona un aliment o plat de la llista");
                        }

                        ui.separator();

                        ui.horizontal(|ui| {
                            if ui.button("➕ Afegir a l'Àpat").clicked() {
                                if !self.add_item_id.is_empty() {
                                    let entries = log.daily_menu.meals.entry(meal).or_insert_with(Vec::new);
                                    entries.push(MealEntry {
                                        id: format!("{}_{}", self.add_item_id, entries.len()),
                                        item_id: self.add_item_id.clone(),
                                        is_dish: self.add_is_dish,
                                        quantity: self.add_quantity,
                                    });
                                    close_modal = true;
                                }
                            }
                            if ui.button("Cancel·lar").clicked() {
                                close_modal = true;
                            }
                        });
                    }
                });

            if close_modal {
                self.show_add_item_dialog = false;
            }
        }

        // Edit Nutritional Goals Modal Dialog
        if self.show_edit_goals_modal {
            let mut close_modal = false;
            egui::Window::new("⚙️ Personalitzar Objectius Nutricionals")
                .collapsible(false)
                .resizable(false)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(screen_rect.center())
                .default_width(modal_width)
                .max_width(modal_width)
                .max_height(modal_height)
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().max_height(modal_height - 90.0).show(ui, |ui| {
                        ui.label("Objectius i límits recomanats per a la teva nutrició:");
                        ui.separator();

                        ui.horizontal_wrapped(|ui| {
                            ui.label("Kcal Mín:");
                            ui.add(egui::DragValue::new(&mut goals.min_kcal).speed(10.0).range(500.0..=5000.0));
                            ui.label("Màx:");
                            ui.add(egui::DragValue::new(&mut goals.max_kcal).speed(10.0).range(500.0..=6000.0));
                        });

                        ui.horizontal_wrapped(|ui| {
                            ui.label("Greixos % Mín:");
                            ui.add(egui::DragValue::new(&mut goals.min_fat_pct).speed(1.0).range(5.0..=50.0));
                            ui.label("% Màx:");
                            ui.add(egui::DragValue::new(&mut goals.max_fat_pct).speed(1.0).range(10.0..=60.0));
                        });

                        ui.horizontal_wrapped(|ui| {
                            ui.label("Greixos Saturats Màx (g):");
                            ui.add(egui::DragValue::new(&mut goals.max_saturated_fat_g).speed(1.0).range(5.0..=100.0));
                        });

                        ui.horizontal_wrapped(|ui| {
                            ui.label("HdC % Mín:");
                            ui.add(egui::DragValue::new(&mut goals.min_carbs_pct).speed(1.0).range(10.0..=70.0));
                            ui.label("% Màx:");
                            ui.add(egui::DragValue::new(&mut goals.max_carbs_pct).speed(1.0).range(20.0..=80.0));
                        });

                        ui.horizontal_wrapped(|ui| {
                            ui.label("Sucres Màx (% macros):");
                            ui.add(egui::DragValue::new(&mut goals.max_sugar_macro_pct).speed(1.0).range(1.0..=30.0));
                        });

                        ui.horizontal_wrapped(|ui| {
                            ui.label("Proteïna Mín (g):");
                            ui.add(egui::DragValue::new(&mut goals.min_protein_g).speed(5.0).range(30.0..=300.0));
                        });

                        ui.horizontal_wrapped(|ui| {
                            ui.label("Fibra Mín (g):");
                            ui.add(egui::DragValue::new(&mut goals.min_fiber_g).speed(1.0).range(10.0..=100.0));
                            ui.label("Ideal:");
                            ui.add(egui::DragValue::new(&mut goals.ideal_fiber_g).speed(1.0).range(15.0..=120.0));
                        });

                        ui.horizontal_wrapped(|ui| {
                            ui.label("Sal Màx (g):");
                            ui.add(egui::DragValue::new(&mut goals.max_salt_g).speed(0.5).range(1.0..=20.0));
                        });

                        ui.horizontal_wrapped(|ui| {
                            ui.label("Ultraprocessats Màx (% Kcal):");
                            ui.add(egui::DragValue::new(&mut goals.max_ultraprocessed_pct).speed(1.0).range(0.0..=50.0));
                        });

                        ui.horizontal_wrapped(|ui| {
                            ui.label("Càrrega Glucèmica Màx (CG):");
                            ui.add(egui::DragValue::new(&mut goals.max_glycemic_load).speed(5.0).range(20.0..=300.0));
                        });
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("💾 Desar Objectius").clicked() {
                            close_modal = true;
                        }
                        if ui.button("Restablir Defecte").clicked() {
                            *goals = NutritionalGoals::default();
                        }
                    });
                });

            if close_modal {
                self.show_edit_goals_modal = false;
            }
        }
    }

    fn export_daily_log(&mut self, state: &AppState) {
        let log = match state.daily_journal.iter().find(|l| l.date == self.selected_date) {
            Some(l) => l,
            None => return,
        };

        let file_name = format!("registre_diari_{}.html", self.selected_date);
        let html = generate_daily_journal_report_html(state, log);

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Document HTML (PDF)", &["html"])
                .set_file_name(&file_name)
                .set_title(format!("Guardar Registre Diari de {}", self.selected_date))
                .save_file()
            {
                if std::fs::write(&path, &html).is_ok() {
                    self.export_message = Some(format!("💾 Fitxer desat amb èxit a: {}", path.display()));
                } else {
                    self.export_message = Some("❌ Error en escriure el fitxer en disc".into());
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Err(e) = crate::web_utils::web::download_file(&file_name, "text/html", html.as_bytes()) {
                self.export_message = Some(format!("❌ Error en descarregar HTML: {:?}", e));
            } else {
                self.export_message = Some(format!("💾 S'ha descarregat {}!", file_name));
            }
        }
    }
}
