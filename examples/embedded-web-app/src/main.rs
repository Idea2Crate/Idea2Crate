fn main() -> egui_web_sandbox::server::Result<()> {
    egui_web_sandbox::server::serve_from_args(egui_web_sandbox::include_app!())
}
