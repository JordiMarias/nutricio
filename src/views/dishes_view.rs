use egui::Ui;
use crate::models::*;
use crate::storage::AppState;

pub struct DishesView {
    pub show_create_dish_modal: bool,
    pub editing_dish_id: Option<String>,

    // Create / Edit Dish form state
    pub dish_name: String,
    pub dish_desc: String,
    pub dish_items: Vec<DishItem>,
    pub selected_add_ing_id: String,
    pub search_ing_query: String,
    pub add_ing_quantity: f64,
}

impl Default for DishesView {
    fn default() -> Self {
        Self {
            show_create_dish_modal: false,
            editing_dish_id: None,

            dish_name: String::new(),
            dish_desc: String::new(),
            dish_items: Vec::new(),
            selected_add_ing_id: String::new(),
            search_ing_query: String::new(),
            add_ing_quantity: 100.0,
        }
    }
}

impl DishesView {
    pub fn ui(&mut self, ui: &mut Ui, state: &mut AppState) {
        ui.heading("Editor de Plats i Receptes Combinades");
        ui.add_space(8.0);

        let is_mobile = ui.ctx().screen_rect().width() < 750.0 || ui.available_width() < 750.0;

        if is_mobile {
            ui.horizontal(|ui| {
                if ui.button("🍲 Crear Nou Plat").clicked() {
                    self.show_create_dish_modal = true;
                    self.editing_dish_id = None;
                    self.dish_name.clear();
                    self.dish_desc.clear();
                    self.dish_items.clear();
                    self.selected_add_ing_id.clear();
                    self.search_ing_query.clear();
                    self.add_ing_quantity = 100.0;
                }
            });
        } else {
            ui.horizontal(|ui| {
                ui.label("Agrega diversos ingredients per formar plats i calcula automàticament els seus valors nutricionals.");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🍲 Crear Nou Plat / Recepta").clicked() {
                        self.show_create_dish_modal = true;
                        self.editing_dish_id = None;
                        self.dish_name.clear();
                        self.dish_desc.clear();
                        self.dish_items.clear();
                        self.selected_add_ing_id.clear();
                        self.search_ing_query.clear();
                        self.add_ing_quantity = 100.0;
                    }
                });
            });
        }

        ui.separator();

        // Grid of Dishes
        let mut to_edit = None;
        let mut to_delete = None;

        for (idx, dish) in state.dishes.iter().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let nova = dish.derived_nova_group(&state.ingredients);
                    crate::views::menu_planner::draw_nova_badge(ui, nova);
                    ui.heading(&dish.name);
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑 Eliminar").clicked() {
                            to_delete = Some(idx);
                        }
                        if ui.button("✏ Editar").clicked() {
                            to_edit = Some(dish.clone());
                        }
                    });
                });

                if let Some(desc) = &dish.description {
                    ui.label(desc);
                }

                ui.add_space(4.0);

                // Dish Nutrition Summary
                let nut = dish.calculate_total_nutrition(&state.ingredients);
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 180, 80), format!("⚡ {:.0} Kcal", nut.kcal));
                    ui.label(format!("• 🥑 Greixos: {:.1}g (Sat: {:.1}g)", nut.fat_g, nut.saturated_fat_g));
                    ui.label(format!("• 🌾 HdC: {:.1}g (Sucres: {:.1}g)", nut.carbs_g, nut.sugars_g));
                    ui.label(format!("• 🥬 Fibra: {:.1}g", nut.fiber_g));
                    ui.label(format!("• 🥩 Proteïna: {:.1}g", nut.protein_g));
                    ui.label(format!("• 🧂 Sal: {:.2}g", nut.salt_g));
                    ui.colored_label(egui::Color32::from_rgb(130, 220, 130), format!("• 💶 {:.2}€", nut.price_euro));
                });

                ui.add_space(2.0);

                // NOVA Percentages Breakdown
                ui.horizontal_wrapped(|ui| {
                    ui.label("📊 Desglossament NOVA:");
                    let breakdown = dish.nova_breakdown(&state.ingredients);
                    let total_k = nut.kcal;
                    for grp in [NovaGroup::Group1Unprocessed, NovaGroup::Group2ProcessedIngredient, NovaGroup::Group3Processed, NovaGroup::Group4UltraProcessed] {
                        let k = breakdown.get(&grp).copied().unwrap_or(0.0);
                        let pct = if total_k > 0.0 { (k / total_k) * 100.0 } else { 0.0 };
                        crate::views::menu_planner::draw_nova_badge(ui, grp);
                        ui.label(format!("{:.1}% ({:.0} Kcal)", pct, k));
                        ui.add_space(4.0);
                    }
                });

                ui.separator();
                ui.label("Ingredients inclosos:");
                if dish.items.is_empty() {
                    ui.label(" (Cap ingredient especificat)");
                } else {
                    for item in &dish.items {
                        if let Some(ing) = state.ingredients.iter().find(|i| i.id == item.ingredient_id) {
                            let qty_str = match ing.unit_type {
                                UnitType::Per100g => {
                                    if item.quantity.fract() == 0.0 {
                                        format!("{:.0} g", item.quantity)
                                    } else {
                                        format!("{:.1} g", item.quantity)
                                    }
                                }
                                UnitType::PerUnit { .. } => {
                                    if item.quantity.fract() == 0.0 {
                                        format!("{:.0} unitats", item.quantity)
                                    } else {
                                        format!("{:.1} unitats", item.quantity)
                                    }
                                }
                            };
                            ui.label(format!("  • {} - {}", ing.name, qty_str));
                        }
                    }
                }
            });
            ui.add_space(8.0);
        }

        if let Some(dish_to_edit) = to_edit {
            self.show_create_dish_modal = true;
            self.editing_dish_id = Some(dish_to_edit.id);
            self.dish_name = dish_to_edit.name;
            self.dish_desc = dish_to_edit.description.unwrap_or_default();
            self.dish_items = dish_to_edit.items;
            self.selected_add_ing_id.clear();
            self.search_ing_query.clear();
            self.add_ing_quantity = 100.0;
        }

        if let Some(del_idx) = to_delete {
            state.dishes.remove(del_idx);
        }

        // MODAL: Create / Edit Dish
        if self.show_create_dish_modal {
            let mut close_modal = false;
            let modal_title = if self.editing_dish_id.is_some() {
                "🍲 Editar Plat / Recepta"
            } else {
                "🍲 Crear Nou Plat / Recepta"
            };

            egui::Window::new(modal_title)
                .collapsible(false)
                .resizable(true)
                .default_size([560.0, 600.0])
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Nom del Plat:");
                            ui.text_edit_singleline(&mut self.dish_name);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Descripció:");
                            ui.text_edit_singleline(&mut self.dish_desc);
                        });

                        // Quick accent toolbar for Catalan characters
                        ui.horizontal_wrapped(|ui| {
                            ui.small("Accents:");
                            for c in ["à", "è", "é", "í", "ï", "ò", "ó", "ú", "ü", "ç", "·", "À", "È", "É", "Í", "Ò", "Ó", "Ú", "Ç"] {
                                if ui.small_button(c).clicked() {
                                    self.dish_name.push_str(c);
                                }
                            }
                        });

                        ui.separator();
                        ui.heading("Afegir Ingredients:");

                        ui.horizontal(|ui| {
                            ui.label("🔍 Cercar ingredient:");
                            ui.text_edit_singleline(&mut self.search_ing_query);
                        });

                        ui.add_space(4.0);

                        egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                            let q = self.search_ing_query.to_lowercase();
                            if state.ingredients.is_empty() {
                                ui.label("La base de dades d'aliments està buida. Afegeix ingredients primer.");
                            } else {
                                let mut matched = 0;
                                for ing in &state.ingredients {
                                    if !q.is_empty() 
                                        && !ing.name.to_lowercase().contains(&q) 
                                        && !ing.brand.as_deref().unwrap_or("").to_lowercase().contains(&q) 
                                    {
                                        continue;
                                    }
                                    matched += 1;
                                    let is_sel = self.selected_add_ing_id == ing.id;
                                    ui.horizontal(|ui| {
                                        let nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                                        let ig = ing.get_glycemic_index();
                                        let level = ing.get_glycemic_level();
                                        crate::views::menu_planner::draw_nova_badge(ui, nova);
                                        crate::views::menu_planner::draw_glycemic_badge(ui, level, ig);

                                        let name_display = if let Some(b) = &ing.brand {
                                            format!("{} [{}] ({:.0} Kcal)", ing.name, b, ing.per_unit_nutrition.kcal)
                                        } else {
                                            format!("{} ({:.0} Kcal)", ing.name, ing.per_unit_nutrition.kcal)
                                        };

                                        if ui.selectable_label(is_sel, name_display).clicked() {
                                            self.selected_add_ing_id = ing.id.clone();
                                            self.add_ing_quantity = match ing.unit_type {
                                                UnitType::Per100g => 100.0,
                                                UnitType::PerUnit { .. } => 1.0,
                                            };
                                        }
                                    });
                                }
                                if matched == 0 {
                                    ui.label("No s'ha trobat cap ingredient coincident amb la cerca.");
                                }
                            }
                        });

                        ui.separator();

                        if let Some(ing) = state.ingredients.iter().find(|i| i.id == self.selected_add_ing_id) {
                            ui.strong(format!("Seleccionat: 🥗 {}", ing.name));
                            let (label_text, speed, max_val, unit_str) = match ing.unit_type {
                                UnitType::Per100g => ("Quantitat:", 0.1, 3000.0, "g"),
                                UnitType::PerUnit { .. } => ("Quantitat:", 0.1, 50.0, "unitats"),
                            };
                            ui.horizontal(|ui| {
                                ui.label(label_text);
                                ui.add(
                                    egui::DragValue::new(&mut self.add_ing_quantity)
                                        .speed(speed)
                                        .range(0.01..=max_val)
                                        .max_decimals(2)
                                );
                                ui.label(unit_str);

                                if ui.button("➕ Afegir al Plat").clicked() {
                                    self.dish_items.push(DishItem {
                                        ingredient_id: self.selected_add_ing_id.clone(),
                                        quantity: self.add_ing_quantity,
                                    });
                                }
                            });
                        } else {
                            ui.colored_label(egui::Color32::YELLOW, "👈 Cerca i selecciona un ingredient de la llista superior per afegir-lo");
                        }

                        ui.separator();
                        ui.heading("Ingredients inclosos en aquest plat:");

                        if self.dish_items.is_empty() {
                            ui.label(" (Encara no has afegit cap ingredient al plat)");
                        } else {
                            let mut item_to_remove = None;
                            for (i_idx, item) in self.dish_items.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    if let Some(ing) = state.ingredients.iter().find(|i| i.id == item.ingredient_id) {
                                        let unit_suffix = match ing.unit_type {
                                            UnitType::Per100g => {
                                                if item.quantity.fract() == 0.0 {
                                                    format!("{:.0}g", item.quantity)
                                                } else {
                                                    format!("{:.1}g", item.quantity)
                                                }
                                            }
                                            UnitType::PerUnit { .. } => {
                                                if item.quantity.fract() == 0.0 {
                                                    format!("{:.0} ut", item.quantity)
                                                } else {
                                                    format!("{:.1} ut", item.quantity)
                                                }
                                            }
                                        };
                                        ui.label(format!("• {} ({})", ing.name, unit_suffix));
                                    } else {
                                        ui.label(format!("• Ingredient {} ({:.1})", item.ingredient_id, item.quantity));
                                    }
                                    if ui.small_button("❌").clicked() {
                                        item_to_remove = Some(i_idx);
                                    }
                                });
                            }
                            if let Some(rem) = item_to_remove {
                                self.dish_items.remove(rem);
                            }

                            // Dish Preview
                            let temp_dish = Dish {
                                id: "preview".to_string(),
                                name: self.dish_name.clone(),
                                description: None,
                                items: self.dish_items.clone(),
                                servings: 1.0,
                            };
                            let nut = temp_dish.calculate_total_nutrition(&state.ingredients);
                            let nova = temp_dish.derived_nova_group(&state.ingredients);
                            ui.add_space(4.0);
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        crate::views::menu_planner::draw_nova_badge(ui, nova);
                                        ui.strong(format!("Valors totals del plat: {:.0} Kcal | {:.1}g Prot | {:.1}g Greixos | {:.1}g HdC | {:.2}€",
                                            nut.kcal, nut.protein_g, nut.fat_g, nut.carbs_g, nut.price_euro));
                                    });
                                    ui.add_space(2.0);
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label("📊 Desglossament NOVA:");
                                        let breakdown = temp_dish.nova_breakdown(&state.ingredients);
                                        let total_k = nut.kcal;
                                        for grp in [NovaGroup::Group1Unprocessed, NovaGroup::Group2ProcessedIngredient, NovaGroup::Group3Processed, NovaGroup::Group4UltraProcessed] {
                                            let k = breakdown.get(&grp).copied().unwrap_or(0.0);
                                            let pct = if total_k > 0.0 { (k / total_k) * 100.0 } else { 0.0 };
                                            crate::views::menu_planner::draw_nova_badge(ui, grp);
                                            ui.label(format!("{:.1}% ({:.0} Kcal)", pct, k));
                                            ui.add_space(4.0);
                                        }
                                    });
                                });
                            });
                        }

                        ui.separator();

                        ui.horizontal(|ui| {
                            let save_label = if self.editing_dish_id.is_some() {
                                "💾 Desar Canvis"
                            } else {
                                "💾 Crear Plat"
                            };

                            if ui.button(save_label).clicked() {
                                if !self.dish_name.trim().is_empty() {
                                    let desc_opt = if self.dish_desc.trim().is_empty() { None } else { Some(self.dish_desc.trim().to_string()) };
                                    if let Some(edit_id) = &self.editing_dish_id {
                                        if let Some(dish) = state.dishes.iter_mut().find(|d| &d.id == edit_id) {
                                            dish.name = self.dish_name.trim().to_string();
                                            dish.description = desc_opt;
                                            dish.items = self.dish_items.clone();
                                        }
                                    } else {
                                        let id = format!("dish_{}", state.dishes.len() + 1);
                                        let dish = Dish {
                                            id,
                                            name: self.dish_name.trim().to_string(),
                                            description: desc_opt,
                                            items: self.dish_items.clone(),
                                            servings: 1.0,
                                        };
                                        state.dishes.push(dish);
                                    }
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
                self.show_create_dish_modal = false;
                self.editing_dish_id = None;
            }
        }
    }
}
