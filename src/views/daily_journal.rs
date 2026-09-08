use egui::{Color32, ProgressBar, Ui};
use chrono::{Datelike, Local, NaiveDate};

use crate::exporter::generate_daily_journal_report_html;
use crate::models::*;
use crate::storage::AppState;
use crate::views::menu_planner::{draw_glycemic_badge, draw_nova_badge};

pub struct DailyJournalView {
    pub selected_date: String,
    pub selected_meal: Option<MealType>,
    pub show_add_item_dialog: bool,
    pub show_edit_goals_modal: bool,
    pub show_copy_confirm_dialog: bool,
    pub copy_source_day_idx: usize,
    pub add_is_dish: bool,
    pub add_item_id: String,
    pub add_quantity: f64,
    pub picker_search: String,
    pub export_message: Option<String>,
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
            add_is_dish: false,
            add_item_id: String::new(),
            add_quantity: 100.0,
            picker_search: String::new(),
            export_message: None,
        }
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

        let is_mobile = ui.ctx().screen_rect().width() < 750.0 || ui.available_width() < 750.0;

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

                ui.add_space(3.0);
                ui.horizontal(|ui| {
                    ui.strong(format!("📅 {}", formatted_date));
                    if is_today {
                        ui.colored_label(Color32::from_rgb(46, 125, 50), "(Avui)");
                    }
                });

                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("📋 Copiar Menú Planificat").clicked() {
                        self.show_copy_confirm_dialog = true;
                    }
                    if ui.button("💾 Exportar HTML").clicked() {
                        trigger_export = true;
                    }
                    if ui.button("⚙ Objectius").clicked() {
                        self.show_edit_goals_modal = true;
                    }
                    if ui.button("🗑 Buidar Dia").clicked() {
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
                        ui.colored_label(Color32::from_rgb(46, 125, 50), " [AVUI]");
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
            ui.colored_label(Color32::GREEN, msg);
        }

        ui.separator();

        // Destructure state to avoid borrowing conflicts
        let AppState { ingredients, dishes, weekly_menu, goals, daily_journal } = state;
        let log = &mut daily_journal[log_idx];

        // SECTION 1: Meals of the Selected Day
        ui.group(|ui| {
            ui.heading(format!("Àpats Ingerits &mdash; {}", formatted_date));
            ui.add_space(4.0);

            for meal_type in MealType::all() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(meal_type.name_ca());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn_text = if is_mobile { "+ Afegir" } else { "+ Afegir Aliment / Plat" };
                            if ui.button(btn_text).clicked() {
                                self.selected_meal = Some(meal_type);
                                self.show_add_item_dialog = true;
                                self.add_item_id.clear();
                                self.picker_search.clear();
                                self.add_quantity = 100.0;
                            }
                        });
                    });

                    ui.separator();

                    let entries = log.daily_menu.meals.entry(meal_type).or_insert_with(Vec::new);

                    if entries.is_empty() {
                        ui.label(" (Sense aliments registrats per a aquest àpat)");
                    } else if is_mobile {
                        // Responsive mobile card layout
                        let mut to_remove = None;
                        for (entry_idx, entry) in entries.iter_mut().enumerate() {
                            ui.group(|ui| {
                                if entry.is_dish {
                                    if let Some(dish) = dishes.iter().find(|d| d.id == entry.item_id) {
                                        let nova = dish.derived_nova_group(ingredients);
                                        let ig = dish.derived_glycemic_index(ingredients);
                                        let level = dish.derived_glycemic_level(ingredients);
                                        let cg = dish.calculate_glycemic_load(ingredients) * entry.quantity;
                                        let nut = dish.calculate_total_nutrition(ingredients).scale(entry.quantity);

                                        ui.horizontal(|ui| {
                                            draw_nova_badge(ui, nova);
                                            draw_glycemic_badge(ui, level, ig);
                                            ui.strong(format!("🍲 {}", dish.name));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("🗑").clicked() {
                                                    to_remove = Some(entry_idx);
                                                }
                                            });
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("Racions:");
                                            ui.add(egui::DragValue::new(&mut entry.quantity).speed(0.1).range(0.1..=20.0).max_decimals(2));
                                            ui.add_space(6.0);
                                            ui.colored_label(Color32::from_rgb(255, 180, 80), format!("⚡ {:.0} kcal", nut.kcal));
                                            ui.add_space(6.0);
                                            ui.colored_label(Color32::from_rgb(130, 220, 130), format!("🏷 {:.2} €", nut.price_euro));
                                        });

                                        ui.horizontal_wrapped(|ui| {
                                            ui.small(format!("🥩 P: {:.1}g", nut.protein_g));
                                            ui.small(format!("🌾 C: {:.1}g (Sucres: {:.1}g, CG: {:.1})", nut.carbs_g, nut.sugars_g, cg));
                                            ui.small(format!("🥑 G: {:.1}g (Sat: {:.1}g)", nut.fat_g, nut.saturated_fat_g));
                                            ui.small(format!("🌿 Fibra: {:.1}g", nut.fiber_g));
                                            ui.small(format!("🧂 Sal: {:.2}g", nut.salt_g));
                                        });
                                    } else {
                                        ui.label(format!("Plat desconegut ({})", entry.item_id));
                                    }
                                } else {
                                    if let Some(ing) = ingredients.iter().find(|i| i.id == entry.item_id) {
                                        let nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                                        let ig = ing.get_glycemic_index();
                                        let level = ing.get_glycemic_level();
                                        let cg = ing.calculate_glycemic_load(entry.quantity);
                                        let nut = ing.calculate_nutrition(entry.quantity);
                                        let (speed, unit_str, max_range) = match ing.unit_type {
                                            UnitType::Per100g => (0.1, "g", 3000.0),
                                            UnitType::PerUnit { .. } => (0.1, "ut", 50.0),
                                        };

                                        ui.horizontal(|ui| {
                                            draw_nova_badge(ui, nova);
                                            draw_glycemic_badge(ui, level, ig);
                                            ui.strong(format!("🥗 {}", ing.name));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("🗑").clicked() {
                                                    to_remove = Some(entry_idx);
                                                }
                                            });
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("Quantitat:");
                                            ui.add(egui::DragValue::new(&mut entry.quantity).speed(speed).range(0.01..=max_range).max_decimals(2));
                                            ui.small(unit_str);
                                            ui.add_space(6.0);
                                            ui.colored_label(Color32::from_rgb(255, 180, 80), format!("⚡ {:.0} kcal", nut.kcal));
                                            ui.add_space(6.0);
                                            ui.colored_label(Color32::from_rgb(130, 220, 130), format!("🏷 {:.2} €", nut.price_euro));
                                        });

                                        ui.horizontal_wrapped(|ui| {
                                            ui.small(format!("🥩 P: {:.1}g", nut.protein_g));
                                            ui.small(format!("🌾 C: {:.1}g (Sucres: {:.1}g, CG: {:.1})", nut.carbs_g, nut.sugars_g, cg));
                                            ui.small(format!("🥑 G: {:.1}g (Sat: {:.1}g)", nut.fat_g, nut.saturated_fat_g));
                                            ui.small(format!("🌿 Fibra: {:.1}g", nut.fiber_g));
                                            ui.small(format!("🧂 Sal: {:.2}g", nut.salt_g));
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
                        egui::Grid::new(format!("journal_meal_grid_{:?}", meal_type))
                            .striped(true)
                            .spacing([12.0, 6.0])
                            .show(ui, |ui| {
                                ui.strong("Element");
                                ui.strong("Porció / Quantitat");
                                ui.strong("Kcal");
                                ui.strong("Greixos (Sat)");
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
                                            let nova = dish.derived_nova_group(ingredients);
                                            let ig = dish.derived_glycemic_index(ingredients);
                                            let level = dish.derived_glycemic_level(ingredients);
                                            let cg = dish.calculate_glycemic_load(ingredients) * entry.quantity;

                                            ui.horizontal(|ui| {
                                                draw_nova_badge(ui, nova);
                                                draw_glycemic_badge(ui, level, ig);
                                                ui.label(format!("🍲 {}", dish.name));
                                            });

                                            ui.horizontal(|ui| {
                                                ui.add(egui::DragValue::new(&mut entry.quantity).speed(0.1).range(0.1..=20.0).max_decimals(2));
                                                ui.small("racions");
                                            });

                                            let nut = dish.calculate_total_nutrition(ingredients).scale(entry.quantity);
                                            ui.label(format!("{:.0}", nut.kcal));
                                            ui.label(format!("{:.1}g ({:.1}g)", nut.fat_g, nut.saturated_fat_g));
                                            ui.label(format!("{:.1}g ({:.1}g) [CG {:.1}]", nut.carbs_g, nut.sugars_g, cg));
                                            ui.label(format!("{:.1}g", nut.fiber_g));
                                            ui.label(format!("{:.1}g", nut.protein_g));
                                            ui.label(format!("{:.2}g", nut.salt_g));
                                            ui.label(format!("{:.2}€", nut.price_euro));
                                        } else {
                                            ui.label(format!("Plat desconegut ({})", entry.item_id));
                                            for _ in 0..8 {
                                                ui.label("-");
                                            }
                                        }
                                    } else {
                                        if let Some(ing) = ingredients.iter().find(|i| i.id == entry.item_id) {
                                            let nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                                            let ig = ing.get_glycemic_index();
                                            let level = ing.get_glycemic_level();
                                            let cg = ing.calculate_glycemic_load(entry.quantity);

                                            ui.horizontal(|ui| {
                                                draw_nova_badge(ui, nova);
                                                draw_glycemic_badge(ui, level, ig);
                                                ui.label(format!("🥗 {}", ing.name));
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
                                            for _ in 0..8 {
                                                ui.label("-");
                                            }
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
                ui.add_space(4.0);
            }
        });

        ui.add_space(12.0);

        // SECTION 2: Nutritional Dashboard & Goals (Full width underneath meals)
        ui.group(|ui| {
            ui.heading(format!("Resum Nutricional Real del Dia ({})", formatted_date));
            ui.add_space(6.0);

            let day_menu = &log.daily_menu;
            let total_nut = day_menu.calculate_total_nutrition(ingredients, dishes);

            ui.group(|ui| {
                ui.strong("⚡ Valor Energètic Totals");
                let kcal_ratio = (total_nut.kcal / goals.max_kcal).min(1.0);
                let kcal_color = if total_nut.kcal >= goals.min_kcal && total_nut.kcal <= goals.max_kcal {
                    Color32::from_rgb(46, 125, 50)
                } else if total_nut.kcal < goals.min_kcal {
                    Color32::from_rgb(245, 124, 0)
                } else {
                    Color32::from_rgb(198, 40, 40)
                };

                ui.add(ProgressBar::new(kcal_ratio as f32).fill(kcal_color).text(format!("{:.0} / {:.0} Kcal", total_nut.kcal, goals.max_kcal)));
                if total_nut.kcal < goals.min_kcal {
                    ui.colored_label(Color32::from_rgb(245, 124, 0), format!("⚠ Falten {:.0} Kcal per arribar al mínim recomanat ({:.0} Kcal)", goals.min_kcal - total_nut.kcal, goals.min_kcal));
                } else if total_nut.kcal > goals.max_kcal {
                    ui.colored_label(Color32::from_rgb(198, 40, 40), format!("⚠ Superat el màxim recomanat per {:.0} Kcal", total_nut.kcal - goals.max_kcal));
                } else {
                    ui.colored_label(Color32::from_rgb(46, 125, 50), "✅ Valor energètic dins del rang recomanat");
                }
            });

            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("📊 Repartiment de Macronutrients (% de Kcal)");
                let total_g = total_nut.total_macro_grams();
                let total_mkcal = total_nut.total_macro_kcal();
                ui.label(format!("Total Kcal de macronutrients: {:.0} Kcal ({:.1} g totals)", total_mkcal, total_g));

                // Fat % & g
                let fat_p = total_nut.fat_pct();
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("• Greixos: {:.1}% ({:.1}g) (Rec: {:.0}-{:.0}%)", fat_p, total_nut.fat_g, goals.min_fat_pct, goals.max_fat_pct));
                    if fat_p >= goals.min_fat_pct && fat_p <= goals.max_fat_pct {
                        ui.colored_label(Color32::GREEN, "OK");
                    } else {
                        ui.colored_label(Color32::YELLOW, "Ajustar");
                    }
                });

                // Carbs % & g
                let carb_p = total_nut.carbs_pct();
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("• Hidrats Carboni: {:.1}% ({:.1}g) (Rec: {:.0}-{:.0}%)", carb_p, total_nut.carbs_g, goals.min_carbs_pct, goals.max_carbs_pct));
                    if carb_p >= goals.min_carbs_pct && carb_p <= goals.max_carbs_pct {
                        ui.colored_label(Color32::GREEN, "OK");
                    } else {
                        ui.colored_label(Color32::YELLOW, "Ajustar");
                    }
                });

                // Protein % & g
                let prot_p = total_nut.protein_pct();
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("• Proteïnes: {:.1}% ({:.1}g) (Rec: {:.0}-{:.0}% | >{:.0}g)", prot_p, total_nut.protein_g, goals.min_protein_pct, goals.max_protein_pct, goals.min_protein_g));
                    if total_nut.protein_g >= goals.min_protein_g {
                        ui.colored_label(Color32::GREEN, "OK");
                    } else {
                        ui.colored_label(Color32::RED, "Manca proteïna");
                    }
                });
            });

            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("📈 Índex i Càrrega Glucèmica (CG)");
                let total_cg = day_menu.calculate_total_glycemic_load(ingredients, dishes);
                let cg_ratio = (total_cg / goals.max_glycemic_load).min(1.0);
                let cg_color = if total_cg <= goals.max_glycemic_load {
                    Color32::from_rgb(46, 125, 50)
                } else {
                    Color32::from_rgb(198, 40, 40)
                };

                ui.add(ProgressBar::new(cg_ratio as f32).fill(cg_color).text(format!("CG Diària: {:.1} / {:.0}", total_cg, goals.max_glycemic_load)));
                if total_cg <= goals.max_glycemic_load {
                    ui.colored_label(Color32::from_rgb(46, 125, 50), "✅ Càrrega Glucèmica sota el límit recomanat");
                } else {
                    ui.colored_label(Color32::from_rgb(198, 40, 40), format!("⚠ Càrrega Glucèmica elevada ({:.1}). Redueix refinats i sucres.", total_cg));
                }
            });

            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("🏷 Classificació i Qualitat NOVA");
                let nova_map = day_menu.nova_breakdown(ingredients, dishes);
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
                    ui.colored_label(Color32::from_rgb(198, 40, 40), format!("⚠ Alt contingut d'Ultraprocessats ({:.1}% de Kcal diàries)", ultra_pct));
                } else {
                    ui.colored_label(Color32::from_rgb(46, 125, 50), "✅ Dieta neta d'ultraprocessats");
                }
            });

            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("⚠ Límits i Control d'Ingredients");
                // Sat Fat
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("• Greixos Saturats: {:.1} g (Màx: {:.0}g)", total_nut.saturated_fat_g, goals.max_saturated_fat_g));
                    if total_nut.saturated_fat_g <= goals.max_saturated_fat_g {
                        ui.colored_label(Color32::GREEN, "OK");
                    } else {
                        ui.colored_label(Color32::RED, "EXCEDIT");
                    }
                });

                // Sugars
                let sugar_p = total_nut.sugar_pct_of_macros();
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("• Sucres: {:.1}% de macros (Màx: {:.0}%)", sugar_p, goals.max_sugar_macro_pct));
                    if sugar_p <= goals.max_sugar_macro_pct {
                        ui.colored_label(Color32::GREEN, "OK");
                    } else {
                        ui.colored_label(Color32::RED, "EXCEDIT");
                    }
                });

                // Fiber
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("• Fibra Alimentària: {:.1} g (Mín: {:.0}g | Ideal: {:.0}g)", total_nut.fiber_g, goals.min_fiber_g, goals.ideal_fiber_g));
                    if total_nut.fiber_g >= goals.ideal_fiber_g {
                        ui.colored_label(Color32::GREEN, "Excel·lent");
                    } else if total_nut.fiber_g >= goals.min_fiber_g {
                        ui.colored_label(Color32::GREEN, "Acceptable");
                    } else {
                        ui.colored_label(Color32::YELLOW, "Insuficient");
                    }
                });

                // Salt
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("• Sal: {:.2} g (Màx: {:.0}g)", total_nut.salt_g, goals.max_salt_g));
                    if total_nut.salt_g <= goals.max_salt_g {
                        ui.colored_label(Color32::GREEN, "OK");
                    } else {
                        ui.colored_label(Color32::RED, "EXCEDIT");
                    }
                });
            });

            ui.add_space(4.0);

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.strong("💶 Despesa Real Diària:");
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
                .default_size([420.0, 200.0])
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
                    ui.colored_label(Color32::YELLOW, "Nota: Això substituirà els àpats actuals d'aquesta data.");
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

            egui::Window::new(format!("Afegir Element a: {}", meal.name_ca()))
                .collapsible(false)
                .resizable(true)
                .default_size([550.0, 480.0])
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(!self.add_is_dish, "🥗 Aliments Individuals").clicked() {
                            self.add_is_dish = false;
                            self.add_item_id.clear();
                        }
                        if ui.selectable_label(self.add_is_dish, "🍲 Plats / Receptes").clicked() {
                            self.add_is_dish = true;
                            self.add_item_id.clear();
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("🔍 Cercar:");
                        ui.text_edit_singleline(&mut self.picker_search);
                    });

                    ui.add_space(4.0);

                    // Scrollable list of items
                    egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                        let q = self.picker_search.to_lowercase();
                        if self.add_is_dish {
                            if dishes.is_empty() {
                                ui.label("Cap plat creat encara. Ves a la pestanya 'Editor de Plats' per crear-ne.");
                            } else {
                                for dish in dishes.iter() {
                                    if !q.is_empty() && !dish.name.to_lowercase().contains(&q) {
                                        continue;
                                    }
                                    let is_sel = self.add_item_id == dish.id;
                                    ui.horizontal(|ui| {
                                        let nova = dish.derived_nova_group(ingredients);
                                        let ig = dish.derived_glycemic_index(ingredients);
                                        let level = dish.derived_glycemic_level(ingredients);
                                        draw_nova_badge(ui, nova);
                                        draw_glycemic_badge(ui, level, ig);
                                        let nut = dish.calculate_total_nutrition(ingredients);
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
                                    if !q.is_empty() && !ing.name.to_lowercase().contains(&q) && !ing.brand.as_deref().unwrap_or("").to_lowercase().contains(&q) {
                                        continue;
                                    }
                                    let is_sel = self.add_item_id == ing.id;
                                    ui.horizontal(|ui| {
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
                                ui.horizontal(|ui| {
                                    ui.label("Racions:");
                                    ui.add(egui::DragValue::new(&mut self.add_quantity).speed(0.1).range(0.1..=20.0));
                                });
                            }
                        } else {
                            if let Some(ing) = ingredients.iter().find(|i| i.id == self.add_item_id) {
                                ui.strong(format!("Seleccionat: 🥗 {}", ing.name));
                                let (label_text, speed, max_val) = match ing.unit_type {
                                    UnitType::Per100g => ("Quantitat (g):", 0.1, 3000.0),
                                    UnitType::PerUnit { .. } => ("Nombre d'unitats:", 0.1, 50.0),
                                };
                                ui.horizontal(|ui| {
                                    ui.label(label_text);
                                    ui.add(egui::DragValue::new(&mut self.add_quantity).speed(speed).range(0.01..=max_val).max_decimals(2));
                                });
                            }
                        }
                    } else {
                        ui.colored_label(Color32::YELLOW, "👈 Selecciona un aliment o plat de la llista superior");
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
                .default_size([450.0, 420.0])
                .show(ui.ctx(), |ui| {
                    ui.label("Modifica els límits i rangs recomanats per a l'avaluació de la teva dieta:");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Kcal Mínimes:");
                        ui.add(egui::DragValue::new(&mut goals.min_kcal).speed(10.0).range(500.0..=5000.0));
                        ui.label("Kcal Màximes:");
                        ui.add(egui::DragValue::new(&mut goals.max_kcal).speed(10.0).range(500.0..=6000.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Greixos % Mín:");
                        ui.add(egui::DragValue::new(&mut goals.min_fat_pct).speed(1.0).range(5.0..=50.0));
                        ui.label("% Màx:");
                        ui.add(egui::DragValue::new(&mut goals.max_fat_pct).speed(1.0).range(10.0..=60.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Greixos Saturats Màx (g):");
                        ui.add(egui::DragValue::new(&mut goals.max_saturated_fat_g).speed(1.0).range(5.0..=100.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Hidrats Carboni % Mín:");
                        ui.add(egui::DragValue::new(&mut goals.min_carbs_pct).speed(1.0).range(10.0..=70.0));
                        ui.label("% Màx:");
                        ui.add(egui::DragValue::new(&mut goals.max_carbs_pct).speed(1.0).range(20.0..=80.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Sucres Màx (% de macros):");
                        ui.add(egui::DragValue::new(&mut goals.max_sugar_macro_pct).speed(1.0).range(1.0..=30.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Proteïna Mín (g):");
                        ui.add(egui::DragValue::new(&mut goals.min_protein_g).speed(5.0).range(30.0..=300.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Fibra Mín (g):");
                        ui.add(egui::DragValue::new(&mut goals.min_fiber_g).speed(1.0).range(10.0..=100.0));
                        ui.label("Ideal (g):");
                        ui.add(egui::DragValue::new(&mut goals.ideal_fiber_g).speed(1.0).range(15.0..=120.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Sal Màx (g):");
                        ui.add(egui::DragValue::new(&mut goals.max_salt_g).speed(0.5).range(1.0..=20.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Ultraprocessats Màx (% Kcal):");
                        ui.add(egui::DragValue::new(&mut goals.max_ultraprocessed_pct).speed(1.0).range(0.0..=50.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Càrrega Glucèmica Màx (CG):");
                        ui.add(egui::DragValue::new(&mut goals.max_glycemic_load).speed(5.0).range(20.0..=300.0));
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("💾 Desar Objectius").clicked() {
                            close_modal = true;
                        }
                        if ui.button("Restablir Valors per Defecte").clicked() {
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
