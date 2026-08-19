//! The Contrapunctus Workbench — a interface over the `contrapunctus` library.
//!
//! `docs/ui-spec.md` is the specification and `docs/ui-sketch.html` the picture;
//! this is the program. The roadmap at the end of that document says which parts
//! of it are built.
//!
//! Nothing here reaches for the filesystem, an environment variable, or a
//! thread, because spec 7 requires the same code to run in a browser. The one
//! platform-specific file in this crate is this one, and it is the window.

mod app;
mod catalog;
mod report;
mod score;
mod strip;
mod theme;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
      .with_inner_size([1180.0, 820.0])
      .with_min_inner_size([880.0, 560.0])
      .with_title("Contrapunctus Workbench"),
    ..Default::default()
  };
  eframe::run_native(
    "contrapunctus",
    options,
    Box::new(|_cc| Ok(Box::<app::App>::default())),
  )
}

#[cfg(target_arch = "wasm32")]
fn main() {
  // The web shell is not written yet — the roadmap has it. This exists so that
  // `cargo check --target wasm32-unknown-unknown` compiles the whole interface
  // and reports anything in it that assumes a desktop, which is the only way
  // that assumption stays out.
}
