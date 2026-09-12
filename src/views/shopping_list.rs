use egui::Ui;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::models::*;
use crate::storage::{save_state_to_json, load_state_from_json, AppState};

pub struct ShoppingListView {
    pub exported_json: String,
    pub imported_json: String,
    pub show_export_dialog: bool,
    pub show_import_dialog: bool,
    pub status_message: Option<String>,
    #[allow(dead_code)]
    pub pending_import_tx: Sender<Result<AppState, String>>,
    pub pending_import_rx: Receiver<Result<AppState, String>>,
}

impl Default for ShoppingListView {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            exported_json: String::new(),
            imported_json: String::new(),
            show_export_dialog: false,
            show_import_dialog: false,
            status_message: None,
            pending_import_tx: tx,
            pending_import_rx: rx,
        }
    }
}

impl ShoppingListView {
    pub fn ui(&mut self, ui: &mut Ui, state: &mut AppState) {
        // Process any async loaded state from file picker
        while let Ok(res) = self.pending_import_rx.try_recv() {
            match res {
                Ok(new_state) => {
                    *state = new_state;
                    self.status_message = Some("✅ Dades carregades correctament des del fitxer!".into());
                }
                Err(e) => {
                    self.status_message = Some(format!("❌ {}", e));
                }
            }
        }

        ui.heading("Llista de la Compra i Gestió de Dades");
        ui.add_space(6.0);

        let is_mobile = ui.available_width() < 720.0;

        ui.horizontal_wrapped(|ui| {
            if ui.button("📂 Carregar Fitxer (JSON)").clicked() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .set_title("Carregar Estat")
                        .pick_file()
                    {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            match load_state_from_json(&content) {
                                Ok(new_state) => {
                                    *state = new_state;
                                    self.status_message = Some("✅ Dades carregades amb èxit!".into());
                                }
                                Err(e) => {
                                    self.status_message = Some(format!("❌ Error JSON: {}", e));
                                }
                            }
                        }
                    }
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let tx = self.pending_import_tx.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("JSON", &["json"])
                            .set_title("Carregar Fitxer JSON")
                            .pick_file()
                            .await;
                        if let Some(handle) = file {
                            let bytes = handle.read().await;
                            match String::from_utf8(bytes) {
                                Ok(text) => match load_state_from_json(&text) {
                                    Ok(new_state) => {
                                        let _ = tx.send(Ok(new_state));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Err(format!("Error en el format JSON: {}", e)));
                                    }
                                },
                                Err(e) => {
                                    let _ = tx.send(Err(format!("Fitxer no vàlid: {}", e)));
                                }
                            }
                        }
                    });
                }
            }

            if ui.button("💾 Desar Fitxer (JSON)").clicked() {
                match save_state_to_json(state) {
                    Ok(json) => {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_file_name("nutricio_estat.json")
                                .set_title("Guardar Estat JSON")
                                .save_file()
                            {
                                if std::fs::write(&path, &json).is_ok() {
                                    self.status_message = Some(format!("💾 Fitxer desat a: {}", path.display()));
                                } else {
                                    self.status_message = Some("❌ Error en escriure el fitxer".into());
                                }
                            }
                        }

                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Err(e) = crate::web_utils::web::download_file("nutricio_estat.json", "application/json", json.as_bytes()) {
                                self.status_message = Some(format!("❌ Error en descarregar fitxer: {:?}", e));
                            } else {
                                self.status_message = Some("💾 S'ha descarregat nutricio_estat.json!".into());
                            }
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("❌ Error en serialitzar JSON: {}", e));
                    }
                }
            }

            if ui.button("🖨️ Descarregar Llista (HTML)").clicked() {
                let html = crate::exporter::generate_shopping_list_report_html(state);
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Document HTML", &["html"])
                        .set_file_name("llista_compra_setmanal.html")
                        .set_title("Guardar Llista de la Compra")
                        .save_file()
                    {
                        if std::fs::write(&path, &html).is_ok() {
                            self.status_message = Some(format!("💾 Llista desada a: {}", path.display()));
                        } else {
                            self.status_message = Some("❌ Error en escriure el fitxer".into());
                        }
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    if let Err(e) = crate::web_utils::web::download_file("llista_compra_setmanal.html", "text/html", html.as_bytes()) {
                        self.status_message = Some(format!("❌ Error en descarregar HTML: {:?}", e));
                    } else {
                        self.status_message = Some("💾 S'ha descarregat llista_compra_setmanal.html!".into());
                    }
                }
            }

            if ui.button("📝 Enganxar text JSON").clicked() {
                self.show_import_dialog = true;
                self.imported_json.clear();
                self.status_message = None;
            }
        });

        if let Some(msg) = &self.status_message {
            ui.add_space(4.0);
            ui.colored_label(egui::Color32::GREEN, msg);
        }

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

        // Sort items deterministically by ingredient name to prevent shuffling on UI repaint frames
        let mut sorted_items: Vec<(String, f64)> = ingredient_quantities.into_iter().collect();
        sorted_items.sort_by(|(id_a, _), (id_b, _)| {
            let name_a = state.ingredients.iter().find(|i| &i.id == id_a).map(|i| i.name.as_str()).unwrap_or(id_a);
            let name_b = state.ingredients.iter().find(|i| &i.id == id_b).map(|i| i.name.as_str()).unwrap_or(id_b);
            name_a.cmp(name_b)
        });

        ui.heading("🛒 Llista d'Ingredients Necessaris per la Setmana:");
        ui.add_space(4.0);

        if sorted_items.is_empty() {
            ui.label("El menú setmanal està buit. Afegeix aliments als àpats per generar la llista de la compra.");
        } else {
            if is_mobile {
                // Mobile Cards
                for (ing_id, total_qty) in &sorted_items {
                    if let Some(ing) = state.ingredients.iter().find(|i| i.id == *ing_id) {
                        let nova = ing.nova_group.unwrap_or(NovaGroup::Group1Unprocessed);
                        let qty_str = match ing.unit_type {
                            UnitType::Per100g => format!("{:.0} g", total_qty),
                            UnitType::PerUnit { .. } => format!("{:.1} unitats", total_qty),
                        };
                        let nut = ing.calculate_nutrition(*total_qty);
                        total_weekly_cost += nut.price_euro;

                        ui.group(|ui| {
                            ui.horizontal_wrapped(|ui| {
                                crate::views::menu_planner::draw_nova_badge(ui, nova);
                                ui.strong(&ing.name);
                                ui.add_space(4.0);
                                ui.colored_label(egui::Color32::from_rgb(130, 220, 130), format!("{:.2} €", nut.price_euro));
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.label(format!("Quantitat total: {}", qty_str));
                            });
                        });
                        ui.add_space(2.0);
                    }
                }
            } else {
                // Desktop Grid
                egui::Grid::new("shopping_grid")
                    .striped(true)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        ui.strong("Aliment");
                        ui.strong("Grup NOVA");
                        ui.strong("Quantitat Total Setmanal");
                        ui.strong("Cost Estimat (€)");
                        ui.end_row();

                        for (ing_id, total_qty) in &sorted_items {
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
            }

            ui.separator();
            ui.group(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.heading("💰 Cost Estimat Total Setmanal:");
                    ui.heading(format!("{:.2} €", total_weekly_cost));
                });
            });
        }

        let screen_rect = ui.ctx().screen_rect();
        let modal_width = (screen_rect.width() - 20.0).min(560.0).max(280.0);
        let modal_height = (screen_rect.height() - 40.0).min(650.0).max(300.0);

        // Export Dialog
        if self.show_export_dialog {
            let mut close = false;
            egui::Window::new("💾 Exportar Tota la Base de Dades (JSON)")
                .collapsible(false)
                .resizable(true)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(screen_rect.center())
                .max_width(modal_width)
                .max_height(modal_height)
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label("Pots copiar el següent text JSON per desar la teva configuració, aliments i menús:");
                        ui.code_editor(&mut self.exported_json);

                        ui.separator();
                        if ui.button("Tancar").clicked() {
                            close = true;
                        }
                    });
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
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(screen_rect.center())
                .max_width(modal_width)
                .max_height(modal_height)
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label("Enganxa el text JSON exportat anteriorment:");
                        ui.code_editor(&mut self.imported_json);

                        if let Some(msg) = &self.status_message {
                            ui.label(msg);
                        }

                        ui.separator();
                        ui.horizontal_wrapped(|ui| {
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
                });
            if close {
                self.show_import_dialog = false;
            }
        }
    }
}
