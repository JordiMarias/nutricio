use egui::{Color32, Ui};
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::models::*;
#[allow(unused_imports)]
use crate::scraper::parse_bonpreu_html;
#[cfg(not(target_arch = "wasm32"))]
use crate::scraper::fetch_and_parse_bonpreu_url;
use crate::storage::AppState;

pub struct IngredientsView {
    pub search_query: String,
    pub selected_nova_filter: Option<NovaGroup>,
    pub show_add_manual_modal: bool,
    pub show_bonpreu_modal: bool,
    pub show_edit_modal: bool,
    pub show_db_sync_modal: bool,
    pub editing_ingredient_id: Option<String>,
    
    // Bonpreu Scraping Form
    pub bonpreu_url: String,
    #[allow(dead_code)]
    pub bonpreu_html_paste: String,
    pub scraping_error: Option<String>,
    pub scraping_success: Option<String>,

    // Database Sync / Import / Export (URL or JSON)
    pub db_sync_url: String,
    pub db_json_paste: String,
    pub db_sync_status: Option<String>,
    pub is_syncing: bool,
    pub pending_db_tx: Sender<Result<String, String>>,
    pub pending_db_rx: Receiver<Result<String, String>>,

    // Ingredient Form fields (used for both Add and Edit)
    pub form_name: String,
    pub form_brand: String,
    pub form_kcal: f64,
    pub form_fat: f64,
    pub form_sat_fat: f64,
    pub form_carbs: f64,
    pub form_sugar: f64,
    pub form_fiber: f64,
    pub form_protein: f64,
    pub form_salt: f64,
    pub form_price: f64,
    pub form_price_per_pack: f64,
    pub form_pack_weight: f64,
    pub form_nova: NovaGroup,
    pub form_ig: u8,
    pub form_ingredients_text: String,
}

impl Default for IngredientsView {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            search_query: String::new(),
            selected_nova_filter: None,
            show_add_manual_modal: false,
            show_bonpreu_modal: false,
            show_edit_modal: false,
            show_db_sync_modal: false,
            editing_ingredient_id: None,

            bonpreu_url: "https://www.compraonline.bonpreuesclat.cat/products/bonpreu-arr%C3%B2s-basmati-integral-ecol%C3%B2gic/85330".to_string(),
            bonpreu_html_paste: String::new(),
            scraping_error: None,
            scraping_success: None,

            db_sync_url: "https://raw.githubusercontent.com/JordiMarias/nutricio/main/base_database.json".to_string(),
            db_json_paste: String::new(),
            db_sync_status: None,
            is_syncing: false,
            pending_db_tx: tx,
            pending_db_rx: rx,

            form_name: String::new(),
            form_brand: String::new(),
            form_kcal: 0.0,
            form_fat: 0.0,
            form_sat_fat: 0.0,
            form_carbs: 0.0,
            form_sugar: 0.0,
            form_fiber: 0.0,
            form_protein: 0.0,
            form_salt: 0.0,
            form_price: 0.0,
            form_price_per_pack: 0.0,
            form_pack_weight: 0.0,
            form_nova: NovaGroup::Group1Unprocessed,
            form_ig: 50,
            form_ingredients_text: String::new(),
        }
    }
}


impl IngredientsView {
    pub fn populate_form_from_ingredient(&mut self, ing: &Ingredient) {
        self.form_name = ing.name.clone();
        self.form_brand = ing.brand.clone().unwrap_or_default();
        self.form_kcal = ing.per_unit_nutrition.kcal;
        self.form_fat = ing.per_unit_nutrition.fat_g;
        self.form_sat_fat = ing.per_unit_nutrition.saturated_fat_g;
        self.form_carbs = ing.per_unit_nutrition.carbs_g;
        self.form_sugar = ing.per_unit_nutrition.sugars_g;
        self.form_fiber = ing.per_unit_nutrition.fiber_g;
        self.form_protein = ing.per_unit_nutrition.protein_g;
        self.form_salt = ing.per_unit_nutrition.salt_g;
        self.form_price = ing.per_unit_nutrition.price_euro;
        self.form_price_per_pack = ing.price_per_pack.unwrap_or(0.0);
        self.form_pack_weight = ing.pack_weight_g.unwrap_or(0.0);
        self.form_nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
        self.form_ig = ing.get_glycemic_index();
        self.form_ingredients_text = ing.ingredients_text.clone().unwrap_or_default();
    }

    pub fn reset_form(&mut self) {
        self.form_name.clear();
        self.form_brand.clear();
        self.form_kcal = 0.0;
        self.form_fat = 0.0;
        self.form_sat_fat = 0.0;
        self.form_carbs = 0.0;
        self.form_sugar = 0.0;
        self.form_fiber = 0.0;
        self.form_protein = 0.0;
        self.form_salt = 0.0;
        self.form_price = 0.0;
        self.form_price_per_pack = 0.0;
        self.form_pack_weight = 0.0;
        self.form_nova = NovaGroup::Group1Unprocessed;
        self.form_ig = 50;
        self.form_ingredients_text.clear();
    }


    pub fn ui(&mut self, ui: &mut Ui, state: &mut AppState) {
        // Process any async loaded database text (e.g. from online fetch or file picker)
        while let Ok(res) = self.pending_db_rx.try_recv() {
            self.is_syncing = false;
            match res {
                Ok(text) => match state.merge_database_from_json(&text) {
                    Ok((n_ing, n_dish)) => {
                        self.db_sync_status = Some(format!("✅ S'han actualitzat / afegit correctament {} aliments i {} plats!", n_ing, n_dish));
                    }
                    Err(e) => {
                        self.db_sync_status = Some(format!("❌ {}", e));
                    }
                },
                Err(e) => {
                    self.db_sync_status = Some(format!("❌ {}", e));
                }
            }
        }

        ui.heading("🥦 Catàleg d'Aliments");
        ui.add_space(8.0);

        let is_mobile = ui.ctx().screen_rect().width() < 750.0 || ui.available_width() < 750.0;

        if is_mobile {
            ui.horizontal_wrapped(|ui| {
                if ui.button("➕ Afegir Aliment").clicked() {
                    self.reset_form();
                    self.show_add_manual_modal = true;
                }
                if ui.button("🛒 Bonpreu").clicked() {
                    self.show_bonpreu_modal = true;
                    self.scraping_error = None;
                    self.scraping_success = None;
                }
                if ui.button("🌐 Sincronitzar Base de Dades (JSON / URL)").clicked() {
                    self.show_db_sync_modal = true;
                    self.db_sync_status = None;
                }
            });

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut self.search_query);
            });

            ui.add_space(2.0);
            egui::ScrollArea::horizontal()
                .id_salt("mobile_nova_filter")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("NOVA:");
                        if ui.selectable_label(self.selected_nova_filter.is_none(), "Tots").clicked() {
                            self.selected_nova_filter = None;
                        }
                        for group in [NovaGroup::Group1Unprocessed, NovaGroup::Group2ProcessedIngredient, NovaGroup::Group3Processed, NovaGroup::Group4UltraProcessed] {
                            let is_sel = self.selected_nova_filter == Some(group);
                            if ui.selectable_label(is_sel, group.short_name_ca()).clicked() {
                                self.selected_nova_filter = Some(group);
                            }
                        }
                    });
                });
        } else {
            ui.horizontal(|ui| {
                ui.label("🔍 Cercar:");
                ui.text_edit_singleline(&mut self.search_query);

                ui.add_space(16.0);

                ui.label("Filtre NOVA:");
                if ui.selectable_label(self.selected_nova_filter.is_none(), "Tots").clicked() {
                    self.selected_nova_filter = None;
                }
                for group in [NovaGroup::Group1Unprocessed, NovaGroup::Group2ProcessedIngredient, NovaGroup::Group3Processed, NovaGroup::Group4UltraProcessed] {
                    let is_sel = self.selected_nova_filter == Some(group);
                    if ui.selectable_label(is_sel, group.short_name_ca()).clicked() {
                        self.selected_nova_filter = Some(group);
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🌐 Sincronitzar Base de Dades (JSON / URL)").clicked() {
                        self.show_db_sync_modal = true;
                        self.db_sync_status = None;
                    }
                    if ui.button("🛒 Importar de Bonpreu").clicked() {
                        self.show_bonpreu_modal = true;
                        self.scraping_error = None;
                        self.scraping_success = None;
                    }
                    if ui.button("➕ Afegir Aliment Manual").clicked() {
                        self.reset_form();
                        self.show_add_manual_modal = true;
                    }
                });
            });
        }

        ui.separator();

        // Ingredients Table / Card View
        let query = self.search_query.to_lowercase();
        let mut to_delete = None;
        let mut to_edit = None;

        if is_mobile {
            // Mobile Card View
            for (idx, ing) in state.ingredients.iter().enumerate() {
                if !query.is_empty() 
                    && !matches_search(&ing.name, &query) 
                    && !matches_search(ing.brand.as_deref().unwrap_or(""), &query) 
                {
                    continue;
                }
                if let Some(nova_filter) = self.selected_nova_filter {
                    if ing.nova_group != Some(nova_filter) {
                        continue;
                    }
                }

                let nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                let ig = ing.get_glycemic_index();
                let level = ing.get_glycemic_level();

                ui.group(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        crate::views::menu_planner::draw_nova_badge(ui, nova);
                        crate::views::menu_planner::draw_glycemic_badge(ui, level, ig);
                        ui.strong(&ing.name);
                        if let Some(b) = &ing.brand {
                            ui.small(format!("({})", b));
                        }
                    });

                    ui.horizontal_wrapped(|ui| {
                        ui.colored_label(Color32::from_rgb(255, 180, 80), format!("⚡ {:.0} kcal / 100g", ing.per_unit_nutrition.kcal));
                        if let Some(price_pack) = ing.price_per_pack {
                            ui.colored_label(Color32::from_rgb(130, 220, 130), format!("🏷 {:.2}€ / pack", price_pack));
                        } else {
                            ui.colored_label(Color32::from_rgb(130, 220, 130), format!("🏷 {:.2}€ / 100g", ing.per_unit_nutrition.price_euro));
                        }
                        if ui.small_button("✏").clicked() {
                            to_edit = Some(ing.clone());
                        }
                        if ui.small_button("🗑").clicked() {
                            to_delete = Some(idx);
                        }
                    });

                    ui.horizontal_wrapped(|ui| {
                        ui.small(format!("🥩 P: {:.1}g", ing.per_unit_nutrition.protein_g));
                        ui.small(format!("🌾 C: {:.1}g (Sucres: {:.1}g)", ing.per_unit_nutrition.carbs_g, ing.per_unit_nutrition.sugars_g));
                        ui.small(format!("🥑 G: {:.1}g (Sat: {:.1}g)", ing.per_unit_nutrition.fat_g, ing.per_unit_nutrition.saturated_fat_g));
                        ui.small(format!("🌿 Fibra: {:.1}g", ing.per_unit_nutrition.fiber_g));
                        ui.small(format!("🧂 Sal: {:.2}g", ing.per_unit_nutrition.salt_g));
                    });
                });
                ui.add_space(3.0);
            }
        } else {
            // Desktop Grid View
            egui::Grid::new("ingredients_grid")
                .striped(true)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.strong("Nom / Marca");
                    ui.strong("NOVA / IG");
                    ui.strong("Kcal (100g/ml)");
                    ui.strong("Greixos (Sat)");
                    ui.strong("HdC (Sucres)");
                    ui.strong("Fibra");
                    ui.strong("Proteïna");
                    ui.strong("Sal");
                    ui.strong("Preu (€)");
                    ui.strong("Accions");
                    ui.end_row();

                    for (idx, ing) in state.ingredients.iter().enumerate() {
                        if !query.is_empty() 
                            && !matches_search(&ing.name, &query) 
                            && !matches_search(ing.brand.as_deref().unwrap_or(""), &query) 
                        {
                            continue;
                        }
                        if let Some(nova_filter) = self.selected_nova_filter {
                            if ing.nova_group != Some(nova_filter) {
                                continue;
                            }
                        }

                        // Name + Brand
                        ui.vertical(|ui| {
                            ui.label(&ing.name);
                            if let Some(b) = &ing.brand {
                                ui.small(format!("Marca: {}", b));
                            }
                        });

                        // NOVA Badge & IG Badge
                        let nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                        let ig = ing.get_glycemic_index();
                        let level = ing.get_glycemic_level();
                        ui.horizontal(|ui| {
                            crate::views::menu_planner::draw_nova_badge(ui, nova);
                            crate::views::menu_planner::draw_glycemic_badge(ui, level, ig);
                        });

                        // Kcal
                        ui.label(format!("{:.0}", ing.per_unit_nutrition.kcal));

                        // Fat
                        ui.label(format!("{:.1}g ({:.1}g)", ing.per_unit_nutrition.fat_g, ing.per_unit_nutrition.saturated_fat_g));

                        // Carbs
                        ui.label(format!("{:.1}g ({:.1}g)", ing.per_unit_nutrition.carbs_g, ing.per_unit_nutrition.sugars_g));

                        // Fiber
                        ui.label(format!("{:.1}g", ing.per_unit_nutrition.fiber_g));

                        // Protein
                        ui.label(format!("{:.1}g", ing.per_unit_nutrition.protein_g));

                        // Salt
                        ui.label(format!("{:.2}g", ing.per_unit_nutrition.salt_g));

                        // Price
                        if let Some(price_pack) = ing.price_per_pack {
                            ui.label(format!("{:.2}€ / pack", price_pack));
                        } else {
                            ui.label(format!("{:.2}€", ing.per_unit_nutrition.price_euro));
                        }

                        ui.horizontal(|ui| {
                            if ui.small_button("✏").clicked() {
                                to_edit = Some(ing.clone());
                            }
                            if ui.small_button("🗑").clicked() {
                                to_delete = Some(idx);
                            }
                        });

                        ui.end_row();
                    }
                });
        }

        if let Some(ing_to_edit) = to_edit {
            self.editing_ingredient_id = Some(ing_to_edit.id.clone());
            self.populate_form_from_ingredient(&ing_to_edit);
            self.show_edit_modal = true;
        }

        if let Some(del_idx) = to_delete {
            state.ingredients.remove(del_idx);
        }

        let screen_rect = ui.ctx().screen_rect();
        let modal_width = (screen_rect.width() - 20.0).min(560.0).max(280.0);
        let modal_height = (screen_rect.height() - 40.0).min(650.0).max(300.0);

        // MODAL 1: Bonpreu Scraper
        if self.show_bonpreu_modal {
            let mut close_modal = false;
            egui::Window::new("🛒 Importar Aliment de Bonpreu Online")
                .collapsible(false)
                .resizable(true)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(screen_rect.center())
                .max_width(modal_width)
                .max_height(modal_height)
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label("Introdueix l'enllaç del producte de la botiga online de Bonpreu:");
                        ui.text_edit_singleline(&mut self.bonpreu_url);

                        ui.add_space(8.0);

                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if ui.button("⚡ Descarregar i Carregar Dades Automàticament").clicked() {
                                match fetch_and_parse_bonpreu_url(&self.bonpreu_url) {
                                    Ok(product) => {
                                        let id = state.generate_unique_ingredient_id("bonpreu");
                                        let ing = Ingredient {
                                            id,
                                            name: product.name.clone(),
                                            brand: product.brand,
                                            source_url: Some(product.source_url),
                                            unit_type: UnitType::Per100g,
                                            per_unit_nutrition: product.nutrition_100g,
                                            price_per_pack: product.price,
                                            pack_weight_g: product.pack_weight_g,
                                            nova_group: Some(product.nova_group),
                                            glycemic_index: Some(product.glycemic_index),
                                            ingredients_text: product.ingredients_text,
                                        };
                                        state.ingredients.push(ing);
                                        self.scraping_success = Some(format!("Afegit correctament: {}", product.name));
                                        self.scraping_error = None;
                                    }
                                    Err(err) => {
                                        self.scraping_error = Some(err);
                                        self.scraping_success = None;
                                    }
                                }
                            }
                        }

                        #[cfg(target_arch = "wasm32")]
                        {
                            ui.label("En mode Web (WASM), si la petició directa és d'un altre domini, pots enganxar el codi HTML de la pàgina del producte:");
                            ui.code_editor(&mut self.bonpreu_html_paste);

                            if ui.button("Parsejar HTML de Bonpreu").clicked() {
                                match parse_bonpreu_html(&self.bonpreu_html_paste, &self.bonpreu_url) {
                                    Ok(product) => {
                                        let id = state.generate_unique_ingredient_id("bonpreu");
                                        let ing = Ingredient {
                                            id,
                                            name: product.name.clone(),
                                            brand: product.brand,
                                            source_url: Some(product.source_url),
                                            unit_type: UnitType::Per100g,
                                            per_unit_nutrition: product.nutrition_100g,
                                            price_per_pack: product.price,
                                            pack_weight_g: product.pack_weight_g,
                                            nova_group: Some(product.nova_group),
                                            glycemic_index: Some(product.glycemic_index),
                                            ingredients_text: product.ingredients_text,
                                        };

                                        state.ingredients.push(ing);
                                        self.scraping_success = Some(format!("Afegit correctament: {}", product.name));
                                        self.scraping_error = None;
                                    }
                                    Err(err) => {
                                        self.scraping_error = Some(err);
                                    }
                                }
                            }
                        }

                        if let Some(msg) = &self.scraping_success {
                            ui.colored_label(Color32::GREEN, msg);
                        }
                        if let Some(err) = &self.scraping_error {
                            ui.colored_label(Color32::RED, err);
                        }

                        ui.separator();
                        if ui.button("Tancar").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if close_modal {
                self.show_bonpreu_modal = false;
            }
        }

        // MODAL 2: Add Manual Ingredient
        if self.show_add_manual_modal {
            let mut close_modal = false;
            egui::Window::new("➕ Afegir Aliment Manualment")
                .collapsible(false)
                .resizable(true)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(screen_rect.center())
                .max_width(modal_width)
                .max_height(modal_height)
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.render_ingredient_form(ui);

                        ui.separator();

                        ui.horizontal_wrapped(|ui| {
                            if ui.button("💾 Desar Aliment").clicked() {
                                if !self.form_name.trim().is_empty() {
                                    let id = state.generate_unique_ingredient_id("manual");
                                    let brand_opt = if self.form_brand.trim().is_empty() { None } else { Some(self.form_brand.trim().to_string()) };
                                    let ing_text_opt = if self.form_ingredients_text.trim().is_empty() { None } else { Some(self.form_ingredients_text.trim().to_string()) };
                                    let price_pack_opt = if self.form_price_per_pack > 0.0 { Some(self.form_price_per_pack) } else { None };
                                    let weight_pack_opt = if self.form_pack_weight > 0.0 { Some(self.form_pack_weight) } else { None };

                                    let mut nut = NutritionalInfo {
                                        kcal: self.form_kcal,
                                        fat_g: self.form_fat,
                                        saturated_fat_g: self.form_sat_fat,
                                        carbs_g: self.form_carbs,
                                        sugars_g: self.form_sugar,
                                        fiber_g: self.form_fiber,
                                        protein_g: self.form_protein,
                                        salt_g: self.form_salt,
                                        price_euro: self.form_price,
                                    };

                                    if let (Some(p), Some(w)) = (price_pack_opt, weight_pack_opt) {
                                        if w > 0.0 {
                                            nut.price_euro = (p / w) * 100.0;
                                        }
                                    }

                                    let ing = Ingredient {
                                        id,
                                        name: self.form_name.trim().to_string(),
                                        brand: brand_opt,
                                        source_url: None,
                                        unit_type: UnitType::Per100g,
                                        per_unit_nutrition: nut,
                                        price_per_pack: price_pack_opt,
                                        pack_weight_g: weight_pack_opt,
                                        nova_group: Some(self.form_nova),
                                        glycemic_index: Some(self.form_ig),
                                        ingredients_text: ing_text_opt,
                                    };
                                    state.ingredients.push(ing);
                                    close_modal = true;
                                }
                            }
                            if ui.button("Cancel·lar").clicked() {
                                close_modal = true;
                            }
                        });
                    });
                });

            if close_modal {
                self.show_add_manual_modal = false;
            }
        }

        // MODAL 3: Edit Ingredient
        if self.show_edit_modal {
            let mut close_modal = false;
            egui::Window::new("✏️ Editar Paràmetres de l'Aliment")
                .collapsible(false)
                .resizable(true)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(screen_rect.center())
                .max_width(modal_width)
                .max_height(modal_height)
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.render_ingredient_form(ui);

                        ui.separator();

                        ui.horizontal_wrapped(|ui| {
                            if ui.button("💾 Desar Canvis").clicked() {
                                if let Some(target_id) = &self.editing_ingredient_id {
                                    if let Some(ing) = state.ingredients.iter_mut().find(|i| i.id == *target_id) {
                                        ing.name = self.form_name.trim().to_string();
                                        ing.brand = if self.form_brand.trim().is_empty() { None } else { Some(self.form_brand.trim().to_string()) };
                                        ing.ingredients_text = if self.form_ingredients_text.trim().is_empty() { None } else { Some(self.form_ingredients_text.trim().to_string()) };
                                        ing.price_per_pack = if self.form_price_per_pack > 0.0 { Some(self.form_price_per_pack) } else { None };
                                        ing.pack_weight_g = if self.form_pack_weight > 0.0 { Some(self.form_pack_weight) } else { None };

                                        let mut nut = NutritionalInfo {
                                            kcal: self.form_kcal,
                                            fat_g: self.form_fat,
                                            saturated_fat_g: self.form_sat_fat,
                                            carbs_g: self.form_carbs,
                                            sugars_g: self.form_sugar,
                                            fiber_g: self.form_fiber,
                                            protein_g: self.form_protein,
                                            salt_g: self.form_salt,
                                            price_euro: self.form_price,
                                        };

                                        if let (Some(p), Some(w)) = (ing.price_per_pack, ing.pack_weight_g) {
                                            if w > 0.0 {
                                                nut.price_euro = (p / w) * 100.0;
                                            }
                                        }

                                        ing.per_unit_nutrition = nut;
                                        ing.nova_group = Some(self.form_nova);
                                        ing.glycemic_index = Some(self.form_ig);
                                    }
                                }
                                close_modal = true;
                            }

                            if ui.button("Cancel·lar").clicked() {
                                close_modal = true;
                            }
                        });
                    });
                });

            if close_modal {
                self.show_edit_modal = false;
                self.editing_ingredient_id = None;
            }
        }

        // MODAL 4: Database Sync / Import / Export (URL or JSON)
        if self.show_db_sync_modal {
            let mut close_modal = false;
            egui::Window::new("🌐 Sincronització de Base de Dades (Aliments i Plats)")
                .collapsible(false)
                .resizable(true)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(screen_rect.center())
                .max_width(modal_width)
                .max_height(modal_height)
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.heading("🌐 1. Sincronització Online des d'Internet (URL)");
                        ui.label("Pots posar l'enllaç d'un fitxer JSON obert a internet (ex: GitHub raw) per actualitzar o afegir aliments i plats automàticament sense perdre els teus menús diaris:");
                        
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Enllaç URL:");
                            ui.text_edit_singleline(&mut self.db_sync_url);
                        });

                        ui.add_space(4.0);
                        if self.is_syncing {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Descarregant i processant dades online...");
                            });
                        } else {
                            if ui.button("⚡ Descarregar i Actualitzar des d'aquesta URL").clicked() {
                                self.is_syncing = true;
                                self.db_sync_status = None;
                                let tx = self.pending_db_tx.clone();
                                let url = self.db_sync_url.trim().to_string();

                                #[cfg(target_arch = "wasm32")]
                                {
                                    wasm_bindgen_futures::spawn_local(async move {
                                        match reqwest::get(&url).await {
                                            Ok(resp) => match resp.text().await {
                                                Ok(text) => { let _ = tx.send(Ok(text)); }
                                                Err(e) => { let _ = tx.send(Err(format!("Error en llegir la resposta HTTP: {}", e))); }
                                            },
                                            Err(e) => {
                                                let _ = tx.send(Err(format!("Error en connectar a la URL: {}", e)));
                                            }
                                        }
                                    });
                                }

                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    std::thread::spawn(move || {
                                        match reqwest::blocking::get(&url) {
                                            Ok(resp) => match resp.text() {
                                                Ok(text) => { let _ = tx.send(Ok(text)); }
                                                Err(e) => { let _ = tx.send(Err(format!("Error en llegir la resposta HTTP: {}", e))); }
                                            },
                                            Err(e) => {
                                                let _ = tx.send(Err(format!("Error en connectar a la URL: {}", e)));
                                            }
                                        }
                                    });
                                }
                            }
                        }

                        ui.separator();
                        ui.heading("📂 2. Carregar des de Fitxer JSON Local");
                        ui.label("Tria un fitxer .json del teu ordinador o mòbil amb aliments/plats:");

                        if ui.button("📂 Obrir Fitxer JSON Local...").clicked() {
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("JSON", &["json"])
                                    .set_title("Obrir Base d'Aliments JSON")
                                    .pick_file()
                                {
                                    if let Ok(content) = std::fs::read_to_string(&path) {
                                        let _ = self.pending_db_tx.send(Ok(content));
                                    }
                                }
                            }

                            #[cfg(target_arch = "wasm32")]
                            {
                                let tx = self.pending_db_tx.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    let file = rfd::AsyncFileDialog::new()
                                        .add_filter("JSON", &["json"])
                                        .set_title("Obrir Base d'Aliments JSON")
                                        .pick_file()
                                        .await;
                                    if let Some(handle) = file {
                                        let bytes = handle.read().await;
                                        if let Ok(text) = String::from_utf8(bytes) {
                                            let _ = tx.send(Ok(text));
                                        }
                                    }
                                });
                            }
                        }

                        ui.separator();
                        ui.heading("📋 3. Enganxar Codi JSON Manualment");
                        ui.label("Pots enganxar directament el text JSON aquí:");
                        ui.code_editor(&mut self.db_json_paste);

                        if ui.button("📥 Fusionar JSON Enganxat").clicked() {
                            if !self.db_json_paste.trim().is_empty() {
                                let _ = self.pending_db_tx.send(Ok(self.db_json_paste.trim().to_string()));
                            }
                        }

                        ui.separator();
                        ui.heading("💾 4. Exportar Base d'Aliments i Plats Actuals");
                        ui.label("Exporta el teu catàleg d'aliments i plats actual per compartir-lo o penjar-lo a GitHub:");

                        if ui.button("💾 Descarregar/Guardar base_database.json").clicked() {
                            let export_obj = serde_json::json!({
                                "ingredients": &state.ingredients,
                                "dishes": &state.dishes
                            });
                            if let Ok(json_text) = serde_json::to_string_pretty(&export_obj) {
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("JSON", &["json"])
                                        .set_file_name("base_database.json")
                                        .set_title("Guardar Base d'Aliments")
                                        .save_file()
                                    {
                                        let _ = std::fs::write(&path, json_text);
                                        self.db_sync_status = Some(format!("💾 Fitxer desat a: {}", path.display()));
                                    }
                                }

                                #[cfg(target_arch = "wasm32")]
                                {
                                    let _ = crate::web_utils::web::download_file(
                                        "base_database.json",
                                        "application/json",
                                        json_text.as_bytes()
                                    );
                                    self.db_sync_status = Some("💾 S'ha iniciat la descàrrega de base_database.json!".into());
                                }
                            }
                        }

                        if let Some(status) = &self.db_sync_status {
                            ui.separator();
                            if status.starts_with("✅") || status.starts_with("💾") {
                                ui.colored_label(Color32::GREEN, status);
                            } else {
                                ui.colored_label(Color32::RED, status);
                            }
                        }

                        ui.separator();
                        if ui.button("Tancar").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if close_modal {
                self.show_db_sync_modal = false;
            }
        }
    }

    fn render_ingredient_form(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Nom:");
            ui.text_edit_singleline(&mut self.form_name);
        });
        ui.horizontal_wrapped(|ui| {
            ui.small("Accents:");
            for c in ["à", "è", "é", "í", "ï", "ò", "ó", "ú", "ü", "ç", "·", "À", "È", "É", "Í", "Ò", "Ó", "Ú", "Ç"] {
                if ui.small_button(c).clicked() {
                    self.form_name.push_str(c);
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Marca:");
            ui.text_edit_singleline(&mut self.form_brand);
        });

        ui.separator();
        ui.label("Valors Nutricionals (per 100g / 100ml):");

        ui.columns(2, |cols| {
            cols[0].horizontal_wrapped(|ui| { ui.label("Kcal:"); ui.add(egui::DragValue::new(&mut self.form_kcal).speed(1.0).range(0.0..=900.0)); });
            cols[0].horizontal_wrapped(|ui| { ui.label("Greixos (g):"); ui.add(egui::DragValue::new(&mut self.form_fat).speed(0.1).range(0.0..=100.0)); });
            cols[0].horizontal_wrapped(|ui| { ui.label("Greixos Sat (g):"); ui.add(egui::DragValue::new(&mut self.form_sat_fat).speed(0.1).range(0.0..=100.0)); });
            cols[0].horizontal_wrapped(|ui| { ui.label("HdC (g):"); ui.add(egui::DragValue::new(&mut self.form_carbs).speed(0.1).range(0.0..=100.0)); });

            cols[1].horizontal_wrapped(|ui| { ui.label("Sucres (g):"); ui.add(egui::DragValue::new(&mut self.form_sugar).speed(0.1).range(0.0..=100.0)); });
            cols[1].horizontal_wrapped(|ui| { ui.label("Fibra (g):"); ui.add(egui::DragValue::new(&mut self.form_fiber).speed(0.1).range(0.0..=100.0)); });
            cols[1].horizontal_wrapped(|ui| { ui.label("Proteïna (g):"); ui.add(egui::DragValue::new(&mut self.form_protein).speed(0.1).range(0.0..=100.0)); });
            cols[1].horizontal_wrapped(|ui| { ui.label("Sal (g):"); ui.add(egui::DragValue::new(&mut self.form_salt).speed(0.01).range(0.0..=100.0)); });
        });

        ui.separator();
        ui.label("Preu i Format:");
        ui.horizontal_wrapped(|ui| {
            ui.label("Preu per 100g/ml (€):");
            ui.add(egui::DragValue::new(&mut self.form_price).speed(0.01).range(0.0..=100.0));
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("O bé Preu Paquet (€):");
            ui.add(egui::DragValue::new(&mut self.form_price_per_pack).speed(0.05).range(0.0..=500.0));
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Pes/Volum Paquet (g/ml):");
            ui.add(egui::DragValue::new(&mut self.form_pack_weight).speed(10.0).range(0.0..=10000.0));
        });

        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label("Classificació NOVA:");
            egui::ComboBox::from_label("Grup NOVA")
                .selected_text(self.form_nova.short_name_ca())
                .show_ui(ui, |ui| {
                    for group in [NovaGroup::Group1Unprocessed, NovaGroup::Group2ProcessedIngredient, NovaGroup::Group3Processed, NovaGroup::Group4UltraProcessed] {
                        ui.selectable_value(&mut self.form_nova, group, group.name_ca());
                    }
                });
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("Índex Glucèmic (IG 0-100):");
            ui.add(egui::Slider::new(&mut self.form_ig, 0..=100).text("IG"));
            let level = GlycemicLevel::from_ig(self.form_ig);
            crate::views::menu_planner::draw_glycemic_badge(ui, level, self.form_ig);
        });

        ui.separator();
        ui.label("Llista d'Ingredients (Text):");
        ui.text_edit_singleline(&mut self.form_ingredients_text);
    }
}

