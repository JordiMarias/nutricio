use egui::Ui;
use crate::models::*;
use crate::storage::AppState;

pub struct DishesView {
    pub show_create_dish_modal: bool,

    // Create Dish form state
    pub dish_name: String,
    pub dish_desc: String,
    pub dish_items: Vec<DishItem>,
    pub selected_add_ing_id: String,
    pub add_ing_quantity: f64,
}

impl Default for DishesView {
    fn default() -> Self {
        Self {
            show_create_dish_modal: false,

            dish_name: String::new(),
            dish_desc: String::new(),
            dish_items: Vec::new(),
            selected_add_ing_id: String::new(),
            add_ing_quantity: 100.0,
        }
    }
}

impl DishesView {
    pub fn ui(&mut self, ui: &mut Ui, state: &mut AppState) {
        ui.heading("Editor de Plats i Receptes Combinades");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Agrega diversos ingredients per formar plats i calcula automàticament els seus valors nutricionals.");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🍲 Crear Nou Plat / Recepta").clicked() {
                    self.show_create_dish_modal = true;
                    self.dish_name.clear();
                    self.dish_desc.clear();
                    self.dish_items.clear();
                }
            });
        });

        ui.separator();

        // Grid of Dishes
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut to_delete = None;

            for (idx, dish) in state.dishes.iter().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let nova = dish.derived_nova_group(&state.ingredients);
                        crate::views::menu_planner::draw_nova_badge(ui, nova);
                        ui.heading(&dish.name);
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑️ Eliminar").clicked() {
                                to_delete = Some(idx);
                            }
                        });
                    });

                    if let Some(desc) = &dish.description {
                        ui.label(desc);
                    }

                    ui.add_space(4.0);

                    // Dish Nutrition Summary
                    let nut = dish.calculate_total_nutrition(&state.ingredients);
                    ui.horizontal(|ui| {
                        ui.label(format!("⚡ Kcal: {:.0} | 🥑 Greixos: {:.1}g (Sat: {:.1}g) | 🌾 HdC: {:.1}g (Sucres: {:.1}g) | 🥬 Fibra: {:.1}g | 🥩 Proteïna: {:.1}g | 🧂 Sal: {:.2}g | 💶 Preu: {:.2}€",
                            nut.kcal, nut.fat_g, nut.saturated_fat_g, nut.carbs_g, nut.sugars_g, nut.fiber_g, nut.protein_g, nut.salt_g, nut.price_euro));
                    });

                    ui.separator();
                    ui.label("Ingredients inclosos:");
                    if dish.items.is_empty() {
                        ui.label(" (Cap ingredient especificat)");
                    } else {
                        for item in &dish.items {
                            if let Some(ing) = state.ingredients.iter().find(|i| i.id == item.ingredient_id) {
                                ui.label(format!("  • {} - {:.0}g / unitats", ing.name, item.quantity));
                            }
                        }
                    }
                });
                ui.add_space(8.0);
            }

            if let Some(del_idx) = to_delete {
                state.dishes.remove(del_idx);
            }
        });

        // MODAL: Create Dish
        if self.show_create_dish_modal {
            let mut close_modal = false;

            egui::Window::new("🍲 Crear Nou Plat")
                .collapsible(false)
                .resizable(true)
                .default_size([500.0, 450.0])
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Nom del Plat:");
                        ui.text_edit_singleline(&mut self.dish_name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Descripció:");
                        ui.text_edit_singleline(&mut self.dish_desc);
                    });

                    ui.separator();
                    ui.heading("Ingredients del Plat:");

                    ui.horizontal(|ui| {
                        egui::ComboBox::from_label("Triar Ingredient")
                            .selected_text(
                                state.ingredients.iter().find(|i| i.id == self.selected_add_ing_id)
                                    .map(|i| i.name.as_str()).unwrap_or("Selecciona...")
                            )
                            .show_ui(ui, |ui| {
                                for ing in &state.ingredients {
                                    ui.selectable_value(&mut self.selected_add_ing_id, ing.id.clone(), &ing.name);
                                }
                            });

                        ui.add(egui::DragValue::new(&mut self.add_ing_quantity).speed(1.0).range(1.0..=1000.0));
                        ui.label("g / u");

                        if ui.button("+ Afegir").clicked() {
                            if !self.selected_add_ing_id.is_empty() {
                                self.dish_items.push(DishItem {
                                    ingredient_id: self.selected_add_ing_id.clone(),
                                    quantity: self.add_ing_quantity,
                                });
                            }
                        }
                    });

                    ui.separator();

                    let mut item_to_remove = None;
                    for (i_idx, item) in self.dish_items.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if let Some(ing) = state.ingredients.iter().find(|i| i.id == item.ingredient_id) {
                                ui.label(format!("• {} ({:.0}g)", ing.name, item.quantity));
                            }
                            if ui.small_button("❌").clicked() {
                                item_to_remove = Some(i_idx);
                            }
                        });
                    }
                    if let Some(rem) = item_to_remove {
                        self.dish_items.remove(rem);
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Desar Plat").clicked() {
                            if !self.dish_name.trim().is_empty() {
                                let id = format!("dish_{}", state.dishes.len() + 1);
                                let desc_opt = if self.dish_desc.trim().is_empty() { None } else { Some(self.dish_desc.trim().to_string()) };
                                let dish = Dish {
                                    id,
                                    name: self.dish_name.trim().to_string(),
                                    description: desc_opt,
                                    items: self.dish_items.clone(),
                                    servings: 1.0,
                                };
                                state.dishes.push(dish);
                                close_modal = true;
                            }
                        }
                        if ui.button("Cancel·lar").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if close_modal {
                self.show_create_dish_modal = false;
            }
        }
    }
}
