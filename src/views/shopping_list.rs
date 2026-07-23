use egui::Ui;
use std::collections::HashMap;
use crate::models::*;
use crate::storage::{save_state_to_json, load_state_from_json, AppState};

pub struct ShoppingListView {
    pub exported_json: String,
    pub imported_json: String,
    pub show_export_dialog: bool,
    pub show_import_dialog: bool,
    pub status_message: Option<String>,
}

impl Default for ShoppingListView {
    fn default() -> Self {
        Self {
            exported_json: String::new(),
            imported_json: String::new(),
            show_export_dialog: false,
            show_import_dialog: false,
            status_message: None,
        }
    }
}

impl ShoppingListView {
    pub fn ui(&mut self, ui: &mut Ui, state: &mut AppState) {
        ui.heading("Llista de la Compra i Gestió de Dades");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Generació automàtica dels ingredients necessaris per completar el menú setmanal definit.");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("📥 Importar Dades (JSON)").clicked() {
                    self.show_import_dialog = true;
                    self.imported_json.clear();
                    self.status_message = None;
                }
                if ui.button("💾 Exportar Dades (JSON)").clicked() {
                    if let Ok(json) = save_state_to_json(state) {
                        self.exported_json = json;
                        self.show_export_dialog = true;
                        self.status_message = None;
                    }
                }
            });
        });

        ui.separator();

        // Calculate total weekly shopping list
        let mut ingredient_quantities: HashMap<String, f64> = HashMap::new();
        let mut total_weekly_cost = 0.0;

        for day in &state.weekly_menu.days {
            for entries in day.meals.values() {
                for entry in entries {
                    if entry.is_dish {
                        if let Some(dish) = state.dishes.iter().find(|d| d.id == entry.item_id) {
                            for item in &dish.items {
                                *ingredient_quantities.entry(item.ingredient_id.clone()).or_insert(0.0) += item.quantity * entry.quantity;
                            }
                        }
                    } else {
                        *ingredient_quantities.entry(entry.item_id.clone()).or_insert(0.0) += entry.quantity;
                    }
                }
            }
        }

        ui.heading("🛒 Llista d'Ingredients Necessaris per la Setmana:");
        ui.add_space(4.0);

        if ingredient_quantities.is_empty() {
            ui.label("El menú setmanal està buit. Afaga aliments als àpats per generar la llista de la compra.");
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("shopping_grid")
                    .striped(true)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        ui.strong("Aliment");
                        ui.strong("Grup NOVA");
                        ui.strong("Quantitat Total Setmanal");
                        ui.strong("Cost Estimat (€)");
                        ui.end_row();

                        for (ing_id, total_qty) in &ingredient_quantities {
                            if let Some(ing) = state.ingredients.iter().find(|i| i.id == *ing_id) {
                                ui.label(&ing.name);

                                let nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                                crate::views::menu_planner::draw_nova_badge(ui, nova);

                                let qty_str = match ing.unit_type {
                                    UnitType::Per100g => format!("{:.0} g", total_qty),
                                    UnitType::PerUnit { .. } => format!("{:.1} unitats", total_qty),
                                };
                                ui.label(qty_str);

                                let nut = ing.calculate_nutrition(*total_qty);
                                total_weekly_cost += nut.price_euro;
                                ui.label(format!("{:.2} €", nut.price_euro));

                                ui.end_row();
                            }
                        }
                    });
            });

            ui.separator();
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("💰 Cost Estimat Total Setmanal:");
                    ui.heading(format!("{:.2} €", total_weekly_cost));
                });
            });
        }

        // Export Dialog
        if self.show_export_dialog {
            let mut close = false;
            egui::Window::new("💾 Exportar Tota la Base de Dades (JSON)")
                .collapsible(false)
                .resizable(true)
                .default_size([500.0, 350.0])
                .show(ui.ctx(), |ui| {
                    ui.label("Pots copiar el següent text JSON per desar la teva configuració, aliments i menús:");
                    ui.code_editor(&mut self.exported_json);

                    ui.separator();
                    if ui.button("Tancar").clicked() {
                        close = true;
                    }
                });
            if close {
                self.show_export_dialog = false;
            }
        }

        // Import Dialog
        if self.show_import_dialog {
            let mut close = false;
            egui::Window::new("📥 Importar Dades (JSON)")
                .collapsible(false)
                .resizable(true)
                .default_size([500.0, 350.0])
                .show(ui.ctx(), |ui| {
                    ui.label("Enganxa el text JSON exportat anteriorment:");
                    ui.code_editor(&mut self.imported_json);

                    if let Some(msg) = &self.status_message {
                        ui.label(msg);
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Carregar Estat").clicked() {
                            match load_state_from_json(&self.imported_json) {
                                Ok(new_state) => {
                                    *state = new_state;
                                    self.status_message = Some("✅ Dades carregades amb èxit!".into());
                                    close = true;
                                }
                                Err(err) => {
                                    self.status_message = Some(format!("❌ Error en llegir JSON: {}", err));
                                }
                            }
                        }
                        if ui.button("Cancel·lar").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                self.show_import_dialog = false;
            }
        }
    }
}
