use std::sync::mpsc::{channel, Receiver, Sender};
use egui::{Context, Visuals};
#[allow(unused_imports)]
use crate::storage::{load_state_from_json, save_state_to_json, AppState};
use crate::views::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    DailyJournal,
    MenuPlanner,
    Ingredients,
    Dishes,
    ShoppingList,
}

pub struct NutricioApp {
    pub state: AppState,
    pub active_tab: ActiveTab,
    pub menu_planner_view: MenuPlannerView,
    pub daily_journal_view: DailyJournalView,
    pub ingredients_view: IngredientsView,
    pub dishes_view: DishesView,
    pub shopping_list_view: ShoppingListView,
    pub status_notification: Option<(String, web_time::Instant)>,
    #[allow(dead_code)]
    pub pending_state_tx: Sender<Result<AppState, String>>,
    pub pending_state_rx: Receiver<Result<AppState, String>>,
    #[cfg(target_arch = "wasm32")]
    pub last_saved_state: Option<AppState>,
    #[cfg(target_arch = "wasm32")]
    pub last_autosave_time: web_time::Instant,
}

impl NutricioApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());

        #[cfg(target_arch = "wasm32")]
        let (initial_state, notification) = {
            if let Some(json) = crate::web_utils::web::load_from_local_storage() {
                match load_state_from_json(&json) {
                    Ok(s) => {
                        let notif = if !s.is_empty() {
                            Some(("🔄 Dades restaurades de la memòria del navegador".to_string(), web_time::Instant::now()))
                        } else {
                            None
                        };
                        (s, notif)
                    }
                    Err(_) => (AppState::empty(), None),
                }
            } else {
                (AppState::empty(), None)
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (initial_state, notification) = (AppState::empty(), None);

        let (pending_state_tx, pending_state_rx) = channel();

        Self {
            state: initial_state.clone(),
            active_tab: ActiveTab::DailyJournal,
            menu_planner_view: MenuPlannerView::default(),
            daily_journal_view: DailyJournalView::default(),
            ingredients_view: IngredientsView::default(),
            dishes_view: DishesView::default(),
            shopping_list_view: ShoppingListView::default(),
            status_notification: notification,
            pending_state_tx,
            pending_state_rx,
            #[cfg(target_arch = "wasm32")]
            last_saved_state: Some(initial_state),
            #[cfg(target_arch = "wasm32")]
            last_autosave_time: web_time::Instant::now(),
        }
    }

    pub fn show_notification(&mut self, msg: impl Into<String>) {
        self.status_notification = Some((msg.into(), web_time::Instant::now()));
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
        let tx = self.pending_state_tx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("JSON", &["json"])
                .set_title("Carregar Fitxer JSON de Nutrició")
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
                        let _ = tx.send(Err(format!("El fitxer no és text vàlid: {}", e)));
                    }
                }
            }
        });
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
        match save_state_to_json(&self.state) {
            Ok(json) => {
                if let Err(e) = crate::web_utils::web::download_file("nutricio_estat.json", "application/json", json.as_bytes()) {
                    self.show_notification(format!("❌ Error en descarregar fitxer: {:?}", e));
                } else {
                    self.show_notification("💾 S'ha descarregat nutricio_estat.json!");
                }
            }
            Err(e) => {
                self.show_notification(format!("❌ Error en serialitzar l'estat: {}", e));
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn sync_local_storage(&mut self) {
        if self.last_saved_state.as_ref() != Some(&self.state) {
            if let Ok(json) = save_state_to_json(&self.state) {
                let _ = crate::web_utils::web::save_to_local_storage(&json);
                self.last_saved_state = Some(self.state.clone());
            }
        }
    }
}

impl eframe::App for NutricioApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Process any async loaded states (e.g. from WASM file picker)
        while let Ok(res) = self.pending_state_rx.try_recv() {
            match res {
                Ok(new_state) => {
                    self.state = new_state;
                    #[cfg(target_arch = "wasm32")]
                    self.sync_local_storage();
                    self.show_notification("✅ Fitxer carregat correctament!");
                    ctx.request_repaint();
                }
                Err(err) => {
                    self.show_notification(format!("❌ {}", err));
                    ctx.request_repaint();
                }
            }
        }

        let screen_width = ctx.screen_rect().width();
        let is_mobile = screen_width < 720.0;

        if is_mobile {
            ctx.style_mut(|s| {
                s.spacing.button_padding = egui::vec2(8.0, 6.0);
                s.spacing.item_spacing = egui::vec2(6.0, 6.0);
            });
        }

        // Top Navigation Bar
        egui::TopBottomPanel::top("top_menu_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            if is_mobile {
                // Mobile layout: Compact header row + Tab row
                ui.horizontal(|ui| {
                    ui.heading("🍏 Nutrició");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.menu_button("⚙ Fitxer / Dades", |ui| {
                            if ui.button("🆕 Nou (Buidar)").clicked() {
                                self.state = AppState::empty();
                                #[cfg(target_arch = "wasm32")]
                                {
                                    let _ = crate::web_utils::web::clear_local_storage();
                                    self.last_saved_state = Some(self.state.clone());
                                }
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
                        });
                    });
                });

                ui.add_space(3.0);
                ui.columns(5, |cols| {
                    if cols[0].selectable_label(self.active_tab == ActiveTab::DailyJournal, "📝 Diari").clicked() {
                        self.active_tab = ActiveTab::DailyJournal;
                    }
                    if cols[1].selectable_label(self.active_tab == ActiveTab::MenuPlanner, "📅 Menús").clicked() {
                        self.active_tab = ActiveTab::MenuPlanner;
                    }
                    if cols[2].selectable_label(self.active_tab == ActiveTab::Ingredients, "🥦 Alim.").clicked() {
                        self.active_tab = ActiveTab::Ingredients;
                    }
                    if cols[3].selectable_label(self.active_tab == ActiveTab::Dishes, "🍲 Plats").clicked() {
                        self.active_tab = ActiveTab::Dishes;
                    }
                    if cols[4].selectable_label(self.active_tab == ActiveTab::ShoppingList, "🛒 Compra").clicked() {
                        self.active_tab = ActiveTab::ShoppingList;
                    }
                });
            } else {
                // Desktop layout
                ui.horizontal(|ui| {
                    ui.menu_button("📄 Fitxer", |ui| {
                        if ui.button("🆕 Nou (Començar de zero)").clicked() {
                            self.state = AppState::empty();
                            #[cfg(target_arch = "wasm32")]
                            {
                                let _ = crate::web_utils::web::clear_local_storage();
                                self.last_saved_state = Some(self.state.clone());
                            }
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
                    });

                    ui.separator();
                    ui.heading("🍏 Nutrició & Despesa Alimentària");
                    ui.add_space(20.0);

                    if ui.selectable_label(self.active_tab == ActiveTab::DailyJournal, "📝 Seguiment Diari").clicked() {
                        self.active_tab = ActiveTab::DailyJournal;
                    }
                    if ui.selectable_label(self.active_tab == ActiveTab::MenuPlanner, "📅 Planificador de Menús").clicked() {
                        self.active_tab = ActiveTab::MenuPlanner;
                    }
                    if ui.selectable_label(self.active_tab == ActiveTab::Ingredients, "🥦 Catàleg d'Aliments").clicked() {
                        self.active_tab = ActiveTab::Ingredients;
                    }
                    if ui.selectable_label(self.active_tab == ActiveTab::Dishes, "🍲 Editor de Plats").clicked() {
                        self.active_tab = ActiveTab::Dishes;
                    }
                    if ui.selectable_label(self.active_tab == ActiveTab::ShoppingList, "🛒 Llista de la Compra & Dades").clicked() {
                        self.active_tab = ActiveTab::ShoppingList;
                    }
                });
            }
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

        // Main Central Panel Content with smooth unified scrolling
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    match self.active_tab {
                        ActiveTab::DailyJournal => {
                            self.daily_journal_view.ui(ui, &mut self.state);
                        }
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
        });

        // Background autosave to localStorage for WASM
        #[cfg(target_arch = "wasm32")]
        {
            if self.last_autosave_time.elapsed().as_millis() > 500 {
                self.sync_local_storage();
                self.last_autosave_time = web_time::Instant::now();
            }
        }
    }
}

