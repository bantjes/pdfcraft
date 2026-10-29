//! Marian PDFCraft: the page thumbnails are always down the left side, in a column of their own
//! with its own scroll bar and the Pages panel's thumbnail size slider. It is the Pages panel
//! ([`super::right_panel`]) drawn there, without its Close button; Pages is then never also the
//! side panel, so Comments, Fields and the rest open on the right beside it.

use crate::{Mode, PdfCraftApp, RightPanel};

/// The column's width when it first shows; it can be dragged between [`MIN_WIDTH`] and [`MAX_WIDTH`].
pub const WIDTH: f32 = 220.0;
pub const MIN_WIDTH: f32 = 150.0;
pub const MAX_WIDTH: f32 = 480.0;

/// Draws the Pages column at the left edge. Call before the side panels and the central panel.
pub fn show(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    if !app.pages_column {
        return;
    }
    if app.right == Some(RightPanel::Pages) {
        app.right = None;
    }
    // Home, Combine files and the expanded All tools view have no document beside them.
    if app.active.is_none() || (app.mode == Mode::AllTools && app.all_tools_full) {
        return;
    }
    let t = crate::theme::Tokens::get(ui.ctx());
    egui::Panel::left(egui::Id::unique("marian-pages"))
        .resizable(true)
        .default_size(WIDTH)
        .size_range(MIN_WIDTH..=MAX_WIDTH)
        .frame(egui::Frame::NONE.fill(t.panel))
        .show(ui, |ui| {
            let side = app.right.replace(RightPanel::Pages);
            app.pages_pinned = true;
            super::right_panel(app, ui);
            app.pages_pinned = false;
            // The thumbnails navigate and select; the side panel keeps its own choice.
            app.right = side;
        });
}
