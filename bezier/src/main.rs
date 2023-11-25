#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    use eframe::{egui::ViewportBuilder, NativeOptions};

    eframe::run_native(
        "Bezier",
        NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size((640.0, 480.0)),
            ..NativeOptions::default()
        },
        Box::new(bezier::Application::create),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "canvas", // hardcode it
                web_options,
                Box::new(bezier::Application::create),
            )
            .await
            .expect("failed to start app");
    });
}
