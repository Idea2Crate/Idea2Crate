fn main() -> egui_web_sandbox::server::Result<()> {
    egui_web_sandbox::server::serve(egui_web_sandbox::include_app!())
}
