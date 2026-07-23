mod app;
mod exporter;
mod models;
mod scraper;
mod storage;
mod views;


use app::NutricioApp;

// Desktop entry point
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("Nutrició & Despesa Alimentària - Planificador Nutricional i NOVA"),
        ..Default::default()
    };

    eframe::run_native(
        "Nutrició & Despesa Alimentària",
        native_options,
        Box::new(|cc| Ok(Box::new(NutricioApp::new(cc)))),
    )
}

// WebAssembly (WASM) entry point
#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async {
        let web_options = eframe::WebOptions::default();
        let runner = eframe::WebRunner::new();

        let document = web_sys::window().expect("Sense window").document().expect("Sense document");

        if let Some(loader) = document.get_element_by_id("centered-loader") {
            loader.remove();
        }

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Sense element the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element no és HtmlCanvasElement");

        runner
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(NutricioApp::new(cc)))),
            )
            .await
            .expect("Error en inicialitzar l'aplicació eframe a WASM");
    });
}
