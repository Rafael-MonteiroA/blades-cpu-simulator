fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Blades CPU Simulator · BLADE-16")
            .with_inner_size([1480.0, 920.0])
            .with_min_inner_size([1050.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Blades CPU Simulator",
        options,
        Box::new(|cc| Ok(Box::new(blades_cpu_simulator::app::BladesApp::new(cc)))),
    )
}
