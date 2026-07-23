use egui::{Context, Visuals};
#[allow(unused_imports)]
use crate::storage::{load_state_from_json, save_state_to_json, AppState};
use crate::views::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    MenuPlanner,
    Ingredients,
    Dishes,
    ShoppingList,
}

pub struct NutricioApp {
    pub state: AppState,
    pub active_tab: ActiveTab,
    pub menu_planner_view: MenuPlannerView,
    pub ingredients_view: IngredientsView,
    pub dishes_view: DishesView,
    pub shopping_list_view: ShoppingListView,
    pub status_notification: Option<(String, std::time::Instant)>,
}

impl NutricioApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());

        Self {
            state: AppState::empty(), // Start from scratch (empty)
            active_tab: ActiveTab::MenuPlanner,
            menu_planner_view: MenuPlannerView::default(),
            ingredients_view: IngredientsView::default(),
            dishes_view: DishesView::default(),
            shopping_list_view: ShoppingListView::default(),
            status_notification: None,
        }
    }

    pub fn show_notification(&mut self, msg: impl Into<String>) {
        self.status_notification = Some((msg.into(), std::time::Instant::now()));
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_title("Carregar Estat de l'Aplicació Nutrició")
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => match load_state_from_json(&content) {
                    Ok(new_state) => {
                        self.state = new_state;
                        self.show_notification("✅ Fitxer carregat correctament!");
                    }
                    Err(e) => {
                        self.show_notification(format!("❌ Error en el format JSON: {}", e));
                    }
                },
                Err(e) => {
                    self.show_notification(format!("❌ Error en llegir el fitxer: {}", e));
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load_file_dialog(&mut self) {
        self.active_tab = ActiveTab::ShoppingList;
        self.shopping_list_view.show_import_dialog = true;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name("nutricio_estat.json")
            .set_title("Guardar Estat de l'Aplicació Nutrició")
            .save_file()
        {
            match save_state_to_json(&self.state) {
                Ok(json) => match std::fs::write(&path, json) {
                    Ok(_) => {
                        self.show_notification("💾 Fitxer desat amb èxit!");
                    }
                    Err(e) => {
                        self.show_notification(format!("❌ Error en escriure el fitxer: {}", e));
                    }
                },
                Err(e) => {
                    self.show_notification(format!("❌ Error en serialitzar l'estat: {}", e));
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save_file_dialog(&mut self) {
        self.active_tab = ActiveTab::ShoppingList;
        if let Ok(json) = save_state_to_json(&self.state) {
            self.shopping_list_view.exported_json = json;
            self.shopping_list_view.show_export_dialog = true;
        }
    }
}

impl eframe::App for NutricioApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Top Menu Bar
        egui::TopBottomPanel::top("top_menu_panel").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.menu_button("📄 Fitxer", |ui| {
                    if ui.button("🆕 Nou (Començar de zero)").clicked() {
                        self.state = AppState::empty();
                        self.show_notification("✨ Iniciat de zero");
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("📂 Carregar Fitxer...").clicked() {
                        self.load_file_dialog();
                        ui.close_menu();
                    }
                    if ui.button("💾 Guardar Fitxer...").clicked() {
                        self.save_file_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🌱 Carregar Dades d'Exemple").clicked() {
                        self.state = AppState::with_seed_data();
                        self.show_notification("🌱 Dades d'exemple carregades");
                        ui.close_menu();
                    }
                });

                ui.separator();
                ui.heading("🍏 Nutrició & Despesa Alimentària");
                ui.add_space(20.0);

                if ui.selectable_label(self.active_tab == ActiveTab::MenuPlanner, "📅 Planificador de Menú").clicked() {
                    self.active_tab = ActiveTab::MenuPlanner;
                }
                if ui.selectable_label(self.active_tab == ActiveTab::Ingredients, "🥦 Catàleg d'Aliments & Bonpreu").clicked() {
                    self.active_tab = ActiveTab::Ingredients;
                }
                if ui.selectable_label(self.active_tab == ActiveTab::Dishes, "🍲 Editor de Plats").clicked() {
                    self.active_tab = ActiveTab::Dishes;
                }
                if ui.selectable_label(self.active_tab == ActiveTab::ShoppingList, "🛒 Llista de la Compra & Dades").clicked() {
                    self.active_tab = ActiveTab::ShoppingList;
                }
            });
            ui.add_space(2.0);
        });

        // Bottom Notification Bar if active
        if let Some((msg, time)) = &self.status_notification {
            if time.elapsed().as_secs() < 4 {
                let msg_clone = msg.clone();
                egui::TopBottomPanel::bottom("notification_panel").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(&msg_clone);
                    });
                });
            }
        }

        // Main Central Panel Content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                ActiveTab::MenuPlanner => {
                    self.menu_planner_view.ui(ui, &mut self.state);
                }
                ActiveTab::Ingredients => {
                    self.ingredients_view.ui(ui, &mut self.state);
                }
                ActiveTab::Dishes => {
                    self.dishes_view.ui(ui, &mut self.state);
                }
                ActiveTab::ShoppingList => {
                    self.shopping_list_view.ui(ui, &mut self.state);
                }
            }
        });
    }
}
