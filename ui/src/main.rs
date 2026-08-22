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
mod audio;
mod catalog;
mod compass;
mod farm;
mod files;
mod glyph;
#[cfg(feature = "midi-out")]
mod midi;
mod palette;
mod report;
mod schedule;
mod score;
mod strip;
mod synth;
mod task;
mod theme;

#[cfg(all(test, feature = "midi-out"))]
mod probe_ports {
  #[test]
  fn list() {
    let out = midir::MidiOutput::new("contrapunctus").expect("a midi client");
    let ports = out.ports();
    eprintln!("{} midi output ports", ports.len());
    for p in &ports {
      eprintln!("  {:?}", out.port_name(p));
    }
  }
}

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
    Box::new(|cc| {
      glyph::install(&cc.egui_ctx);
      Ok(Box::<app::App>::default())
    }),
  )
}

/// The browser entry — spec 7, and the same application.
///
/// Nothing above this line is conditional. The whole interface, the search, the
/// corpus and the synth are one build; what differs is that a window is asked
/// for on one target and a canvas is found on the other. That is the test of
/// whether spec 7's rules were followed, and it is a test that fails loudly:
/// anything reaching for a path, a thread or an environment variable would not
/// compile here at all.
#[cfg(target_arch = "wasm32")]
fn main() {
  use eframe::wasm_bindgen::JsCast as _;

  wasm_bindgen_futures::spawn_local(async {
    let document = web_sys::window().expect("a window").document().expect("a document");
    let canvas = document
      .get_element_by_id("workbench")
      .expect("index.html has a canvas with id `workbench`")
      .dyn_into::<web_sys::HtmlCanvasElement>()
      .expect("`workbench` is a canvas");

    let started = eframe::WebRunner::new()
      .start(canvas, eframe::WebOptions::default(), Box::new(|cc| {
        glyph::install(&cc.egui_ctx);
        Ok(Box::<app::App>::default())
      }))
      .await;

    // Whatever happens, say so on the page rather than in a console nobody has
    // open — the same rule the interface applies to a missing sound card.
    if let Some(el) = document.get_element_by_id("loading") {
      match started {
        Ok(()) => el.remove(),
        Err(e) => el.set_inner_html(&format!("<p>The workbench did not start: {e:?}</p>")),
      }
    }
  });
}
