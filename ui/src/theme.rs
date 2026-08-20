//! Colour, and one rule about it.
//!
//! A voice keeps its colour everywhere it appears — the plan strip, the score's
//! tint, the report. That is the whole of the visual system, and it is what lets
//! spec 1's non-expert follow the subject from lane to lane without being told
//! what a voice is.

use egui::Color32;

/// Voice colours, top voice first. Two sets, because the interface renders in
/// whatever theme the host is in and a colour legible on paper is not legible
/// on ink.
pub fn voice(v: usize, dark: bool) -> Color32 {
  const LIGHT: [Color32; 4] = [
    Color32::from_rgb(0x33, 0x41, 0x7E),
    Color32::from_rgb(0x2E, 0x6F, 0x6B),
    Color32::from_rgb(0x8A, 0x6D, 0x1F),
    Color32::from_rgb(0x7A, 0x35, 0x5C),
  ];
  const DARK: [Color32; 4] = [
    Color32::from_rgb(0x80, 0x90, 0xDE),
    Color32::from_rgb(0x5F, 0xB8, 0xB1),
    Color32::from_rgb(0xCB, 0xA8, 0x4B),
    Color32::from_rgb(0xD0, 0x8C, 0xB4),
  ];
  let set = if dark { DARK } else { LIGHT };
  set[v % set.len()]
}

/// The same colour at low opacity, for an episode's hatch and the score's tint.
pub fn wash(v: usize, dark: bool, alpha: u8) -> Color32 {
  let c = voice(v, dark);
  Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

/// Something is wrong here and the interface is not hiding it.
pub fn warn(dark: bool) -> Color32 {
  if dark { Color32::from_rgb(0xE0, 0x77, 0x6B) } else { Color32::from_rgb(0xB4, 0x34, 0x2A) }
}

/// Something is as it should be.
pub fn good(dark: bool) -> Color32 {
  if dark { Color32::from_rgb(0x5F, 0xBE, 0x86) } else { Color32::from_rgb(0x2F, 0x7A, 0x4F) }
}
