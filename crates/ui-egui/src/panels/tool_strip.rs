//! Marian PDFCraft: the Tools pane, closed, folds to a thin column of tool-set symbols on its
//! side (Preferences ▸ Tools panel side), instead of going away. Its first button opens All tools
//! again; each symbol below opens that tool set, its name showing on hover. Tool sets in the
//! Closet or the Basement ([`crate::closet`]) stay out of it.

use pdfcraft_engine::catalog::TOOL_GROUPS;

use super::side_panels::ToolsSide;
use crate::{LeftPanel, Mode, PdfCraftApp, icons};

/// The column's width: one icon button and its margins.
pub const WIDTH: f32 = 46.0;

/// Draws the column when the Tools pane is closed. Call after the rail and before the panes, so
/// on the right it sits inside the rail.
pub fn show(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    if !app.tool_strip || app.left_open || app.mode == Mode::Read {
        return;
    }
    let t = crate::theme::Tokens::get(ui.ctx());
    let id = egui::Id::unique("marian-tool-strip");
    let (panel, open_icon) = match app.side_panels.tools_side {
        ToolsSide::Left => (egui::Panel::left(id), "panel-left"),
        ToolsSide::Right => (egui::Panel::right(id), "panel-right"),
    };
    let mut chosen = None;
    let mut open_all = false;
    panel
        .resizable(false)
        .exact_size(WIDTH)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(7, 10)).stroke(egui::Stroke::new(1.0, t.divider)))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    if icons::button(ui, open_icon, 32.0, false, tl!("Show all tools")).clicked() {
                        open_all = true;
                    }
                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);
                    for g in TOOL_GROUPS.iter().filter(|g| !app.group_put_away(g.id)) {
                        let on = app.left == LeftPanel::Tool(g.id);
                        // In each tool set's own colour, as in the list; the open one in green.
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
                        resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, tl!(g.label)));
                        let press = crate::theme::Press::track(ui, &resp);
                        let fill = if on { t.current_tool } else { press.fill(egui::Color32::TRANSPARENT, t.hover, t.pressed) };
                        ui.painter().rect_filled(rect, egui::CornerRadius::same(6), fill);
                        let tint = if on { t.on_current_tool } else { egui::Color32::from_rgb(g.hue[0], g.hue[1], g.hue[2]) };
                        icons::paint(ui, rect.translate(press.offset()).shrink(7.0), g.icon, 18.0, tint);
                        if crate::theme::hand(resp).on_hover_text(tl!(g.label)).clicked() {
                            chosen = Some(g.id);
                        }
                    }
                });
        });
    if let Some(id) = chosen {
        app.select_tool(id);
    } else if open_all {
        app.left = LeftPanel::AllTools;
        app.reveal_tools();
    }
}

impl PdfCraftApp {
    /// Marian PDFCraft's own starting choices, applied once over settings carried over from
    /// PdfCraft: the Tools pane on the right.
    pub fn apply_marian_defaults(&mut self) {
        if !self.marian_defaults_applied {
            self.side_panels.tools_side = ToolsSide::Right;
            self.marian_defaults_applied = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(app: PdfCraftApp) -> egui_kittest::Harness<'static, PdfCraftApp> {
        let mut h =
            egui_kittest::Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(|ui, app: &mut PdfCraftApp| show(app, ui), app);
        h.run_steps(2);
        h
    }

    #[test]
    fn a_closed_tools_pane_folds_to_its_symbols_and_they_open_their_tools() {
        use egui_kittest::kittest::Queryable;
        let mut app = PdfCraftApp::new();
        app.tool_strip = true;
        app.closet_sorted = true;
        app.left_open = false;
        let mut h = run(app);
        assert!(h.query_by_label("Scan & OCR").is_none(), "a tool set in the Basement stays out");
        h.get_by_label("Redact a PDF").click();
        h.run_steps(2);
        assert!(h.state().left_open);
        assert_eq!(h.state().left, LeftPanel::Tool("redact"));
    }

    #[test]
    fn the_column_is_off_unless_asked_for_and_gone_while_the_pane_is_open() {
        use egui_kittest::kittest::Queryable;
        let mut app = PdfCraftApp::new();
        app.left_open = false;
        let h = run(app);
        assert!(h.query_by_label("Show all tools").is_none());
        let mut app = PdfCraftApp::new();
        app.tool_strip = true;
        app.left_open = true;
        let h = run(app);
        assert!(h.query_by_label("Show all tools").is_none());
    }

    #[test]
    fn marian_defaults_put_the_tools_on_the_right_once() {
        let mut app = PdfCraftApp::new();
        app.apply_marian_defaults();
        assert_eq!(app.side_panels.tools_side, ToolsSide::Right);
        app.side_panels.tools_side = ToolsSide::Left;
        app.apply_marian_defaults();
        assert_eq!(app.side_panels.tools_side, ToolsSide::Left, "her own later choice stays");
    }
}
