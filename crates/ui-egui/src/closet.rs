//! The Closet and the Basement (Window ▸ Closet…): tool sets, markup tools, side panels and menu
//! items put out of sight without deleting anything. The Closet holds what is used now and then,
//! one click from coming back; the Basement, a tab further away, holds what will probably never
//! be used. A tool set put away leaves the All tools list, the expanded tools view, the Home
//! page, View ▸ Tools, the mode tabs and the menu items that belong only to it; a markup tool
//! leaves the floating tool bar and Add comments; a side panel leaves the right-hand strip and
//! View ▸ Side panels; a menu item leaves its menu. The command palette still finds everything.
//!
//! Everything is named by a key: `group:<tool set id>` (`group:scan`), `tool:<command id>`
//! (`tool:comment.underline`), `panel:<name>` (`panel:layers`) or `menu:<command id>`
//! (`menu:file.word_count`). [`SHIPPED`] is the sort Marian PDFCraft starts with; a menu item
//! that [`SHIPPED`] doesn't name follows its tool sets. `PdfCraftApp::closet` keeps only what was
//! moved since, so it is saved with the settings.

use std::collections::BTreeMap;

use egui::RichText;
use pdfcraft_engine::catalog::TOOL_GROUPS;
use serde_json::{Value, json};

use crate::theme::Tokens;
use crate::tools::comments;
use crate::{PdfCraftApp, RightPanel};

/// Where something lives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shelf {
    InUse,
    Closet,
    Basement,
}

impl Shelf {
    const ALL: [Self; 3] = [Self::InUse, Self::Closet, Self::Basement];

    pub fn name(self) -> &'static str {
        match self {
            Self::InUse => "inUse",
            Self::Closet => "closet",
            Self::Basement => "basement",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|shelf| shelf.name() == s)
    }
}

use Shelf::{Basement, Closet};

/// The sort Marian PDFCraft starts with (Emma's list, 29 October 2026). What she uses: page
/// thumbnails, organize, combine, extract, zoom, sticky notes, highlight, drawing with a pen,
/// Fill & Sign, forms with signature fields, redaction and print.
pub const SHIPPED: &[(&str, Shelf)] = &[
    // Tool sets.
    ("group:export", Closet),
    ("group:create", Closet),
    ("group:edit", Closet),
    ("group:compress", Closet),
    ("group:protect", Closet),
    ("group:stamp", Closet),
    ("group:print_production", Closet),
    ("group:scan", Basement),
    ("group:standards", Basement),
    ("group:measure", Basement),
    ("group:compare", Basement),
    ("group:certificate", Basement),
    ("group:actions", Basement),
    ("group:accessibility", Basement),
    ("group:javascript", Basement),
    ("group:search_index", Basement),
    // Markup tools: Sticky note, Highlight, Draw and Eraser stay.
    ("tool:comment.freetext", Closet),
    ("tool:comment.callout", Closet),
    ("tool:comment.attach", Closet),
    ("tool:comment.underline", Closet),
    ("tool:comment.strikeout", Closet),
    ("tool:comment.squiggly", Closet),
    ("tool:comment.caret", Closet),
    ("tool:comment.replace", Closet),
    ("tool:comment.line", Closet),
    ("tool:comment.arrow", Closet),
    ("tool:comment.square", Closet),
    ("tool:comment.circle", Closet),
    ("tool:comment.polyline", Closet),
    ("tool:comment.polygon", Closet),
    ("tool:comment.cloud", Closet),
    // Side panels: Comments and Form fields stay (Pages is always on the left).
    ("panel:search", Closet),
    ("panel:bookmarks", Closet),
    ("panel:signatures", Closet),
    ("panel:layers", Closet),
    ("panel:attachments", Closet),
    ("panel:compare", Basement),
    ("panel:accessibility", Basement),
    ("panel:tags", Basement),
    // Menu items that belong to no tool set.
    ("menu:file.word_count", Closet),
    ("menu:edit.advanced_search", Closet),
    ("menu:edit.column_select", Closet),
    ("menu:edit.snapshot", Closet),
    ("menu:protect.unlock", Closet),
    ("menu:page.properties", Closet),
    ("menu:bookmark.add", Closet),
    ("menu:bookmark.from_structure", Closet),
    ("menu:bookmark.from_text_styles", Closet),
    ("menu:sign.panel", Closet),
];

/// Menu items that stay: the way into the Closet, and the ones every app must have.
const KEPT_COMMANDS: [&str; 6] = ["window.closet", "app.preferences", "help.about", "file.open", "file.close", "file.save"];

/// The side panels that can be put away: the panel and its name in keys (View ▸ Side panels).
pub(crate) fn panel_name(panel: RightPanel) -> Option<&'static str> {
    crate::menus::PANELS.iter().find(|(p, ..)| *p == panel).map(|(_, name, _)| *name)
}

impl PdfCraftApp {
    /// Where [`SHIPPED`] puts `key`, or for a menu item SHIPPED doesn't name, where its tool
    /// sets are (put away once every tool set it belongs to is).
    fn closet_default(&self, key: &str) -> Shelf {
        if let Some((_, s)) = SHIPPED.iter().find(|(k, _)| *k == key).filter(|_| self.closet_sorted) {
            return *s;
        }
        let Some(id) = key.strip_prefix("menu:") else { return Shelf::InUse };
        // A markup tool's menu item goes with the tool.
        if comments::CommentTool::from_command(id).is_some() {
            let tool = self.shelf(&format!("tool:{id}"));
            if tool != Shelf::InUse {
                return tool;
            }
        }
        let groups: Vec<Shelf> = TOOL_GROUPS
            .iter()
            .filter(|g| g.sections.iter().flat_map(|s| s.items.iter()).any(|i| i.command == id || i.actions.contains(&id)))
            .map(|g| self.shelf(&format!("group:{}", g.id)))
            .collect();
        if groups.is_empty() || groups.contains(&Shelf::InUse) {
            Shelf::InUse
        } else if groups.contains(&Shelf::Closet) {
            Shelf::Closet
        } else {
            Shelf::Basement
        }
    }

    /// Where `key` lives now.
    pub(crate) fn shelf(&self, key: &str) -> Shelf {
        self.closet.get(key).and_then(|s| Shelf::parse(s)).unwrap_or_else(|| self.closet_default(key))
    }

    /// Is `key` put away (in the Closet or the Basement)?
    pub(crate) fn put_away(&self, key: &str) -> bool {
        self.shelf(key) != Shelf::InUse
    }

    /// Is the tool set `id` put away?
    pub(crate) fn group_put_away(&self, id: &str) -> bool {
        self.put_away(&format!("group:{id}"))
    }

    /// Is the markup tool (or other tool entry) run by `command` put away?
    pub(crate) fn tool_put_away(&self, command: &str) -> bool {
        self.put_away(&format!("tool:{command}"))
    }

    /// Is the side panel put away? Pages, always on the left, never is.
    pub(crate) fn panel_put_away(&self, panel: RightPanel) -> bool {
        panel_name(panel).is_some_and(|name| self.put_away(&format!("panel:{name}")))
    }

    /// Is the menu item `command` put away? Items with parameters (recent files, documents)
    /// stay, except a side panel's, which goes with its panel.
    fn menu_put_away(&self, command: &craft_menu::CommandRef) -> bool {
        if command.id == crate::menus::PANEL {
            return crate::menus::panel(command).is_some_and(|p| self.panel_put_away(p));
        }
        if !command.params.is_null() || KEPT_COMMANDS.contains(&command.id.as_str()) {
            return false;
        }
        self.put_away(&format!("menu:{}", command.id))
    }

    /// The menus without what is put away; submenus left empty go too.
    pub(crate) fn prune_menus(&self, bar: &mut craft_menu::MenuBar) {
        if !self.closet_sorted && self.closet.is_empty() {
            return;
        }
        for menu in &mut bar.menus {
            let items = std::mem::take(&mut menu.children);
            let kept = self.prune(items);
            menu.children = craft_menu::tidy(kept);
        }
    }

    fn prune(&self, nodes: Vec<craft_menu::Node>) -> Vec<craft_menu::Node> {
        use craft_menu::Node;
        nodes
            .into_iter()
            .filter_map(|node| match node {
                Node::Item(item) if self.menu_put_away(&item.command) => None,
                Node::Submenu { label, children, role, platforms } => Some(Node::Submenu { label, children: self.prune(children), role, platforms }),
                other => Some(other),
            })
            .collect()
    }

    /// The tool group's markup tools in use, for the floating tool bar's flyout.
    pub(crate) fn markup_tools_in_use(&self, group: usize) -> Vec<comments::CommentTool> {
        comments::GROUPS.get(group).map(|g| g.iter().copied().filter(|t| !self.tool_put_away(t.command())).collect()).unwrap_or_default()
    }

    /// Move `key` to `to`, remembering only what differs from where it would be anyway.
    fn closet_set(&mut self, key: &str, to: Shelf) {
        self.closet.remove(key);
        if self.closet_default(key) != to {
            self.closet.insert(key.to_owned(), to.name().to_owned());
        }
    }

    /// After a move: a side panel put away closes, and a tool set put away leaves the tools pane.
    fn closet_tidy(&mut self) {
        if let Some(p) = self.right
            && p != RightPanel::Pages
            && self.panel_put_away(p)
        {
            self.right = None;
        }
        if let crate::LeftPanel::Tool(id) = self.left
            && self.group_put_away(id)
        {
            self.left = crate::LeftPanel::AllTools;
        }
        if let crate::QuickTool::Comment(t) = self.quick_tool
            && self.tool_put_away(t.command())
        {
            self.quick_tool = crate::QuickTool::Select;
        }
    }

    /// `closet.set {key, shelf}`, `closet.reset {all?}` and `closet.list {shelf?}` for automation;
    /// `None` for other ids.
    pub(crate) fn closet_command(&mut self, id: &str, p: &Value) -> Option<Result<Value, String>> {
        Some(match id {
            "closet.set" => (|| {
                let key =
                    p.get("key").and_then(Value::as_str).ok_or("key is required: group:<id>, tool:<command>, panel:<name> or menu:<command>")?;
                let to = p.get("shelf").and_then(Value::as_str).and_then(Shelf::parse).ok_or("shelf must be inUse, closet or basement")?;
                if !things().iter().any(|t| t.key == key) {
                    return Err(format!("nothing called `{key}` can be put away"));
                }
                self.closet_set(key, to);
                self.closet_tidy();
                Ok(json!({ "key": key, "shelf": to.name() }))
            })(),
            "closet.reset" => {
                let all = p.get("all").and_then(Value::as_bool).unwrap_or(false);
                self.closet_reset(all);
                Ok(self.closet_list(None))
            }
            "closet.list" => (|| {
                let shelf = match p.get("shelf").and_then(Value::as_str) {
                    None => None,
                    Some(s) => Some(Shelf::parse(s).ok_or("shelf must be inUse, closet or basement")?),
                };
                Ok(self.closet_list(shelf))
            })(),
            _ => return None,
        })
    }

    /// Back to the starting sort, or with `all` everything in use.
    fn closet_reset(&mut self, all: bool) {
        self.closet = BTreeMap::new();
        if all {
            let keys: Vec<String> = things().into_iter().map(|t| t.key).collect();
            for key in keys {
                if self.closet_default(&key) != Shelf::InUse {
                    self.closet.insert(key, Shelf::InUse.name().to_owned());
                }
            }
        }
        self.closet_tidy();
    }

    /// The keys on `shelf` (every key put away, by shelf, without one).
    fn closet_list(&self, shelf: Option<Shelf>) -> Value {
        let things = things();
        let on = |s: Shelf| things.iter().filter(|t| self.shelf(&t.key) == s).map(|t| t.key.clone()).collect::<Vec<_>>();
        match shelf {
            Some(s) => json!(on(s)),
            None => json!({ "closet": on(Shelf::Closet), "basement": on(Shelf::Basement) }),
        }
    }
}

// ---------- what can be put away ----------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    ToolSet,
    Markup,
    Panel,
    Menu,
}

impl Kind {
    fn heading(self) -> &'static str {
        match self {
            Self::ToolSet => "Tool sets",
            Self::Markup => "Markup tools",
            Self::Panel => "Side panels",
            Self::Menu => "Menu items",
        }
    }
}

/// One thing that can be put away.
#[derive(Clone, Debug)]
pub struct Thing {
    pub key: String,
    pub kind: Kind,
    /// Its English name (a menu item's whole path).
    pub label: String,
}

/// Every tool set, markup tool, side panel and menu item, in that order.
pub fn things() -> Vec<Thing> {
    let mut out = Vec::new();
    for g in TOOL_GROUPS {
        out.push(Thing { key: format!("group:{}", g.id), kind: Kind::ToolSet, label: g.label.to_owned() });
    }
    for group in comments::GROUPS {
        for tool in group {
            out.push(Thing { key: format!("tool:{}", tool.command()), kind: Kind::Markup, label: tool.label().to_owned() });
        }
    }
    for (_, name, label) in crate::menus::PANELS {
        out.push(Thing { key: format!("panel:{name}"), kind: Kind::Panel, label: (*label).to_owned() });
    }
    let mut seen: Vec<&str> = Vec::new();
    for (path, ids) in crate::menus::MENU_LISTS {
        for &id in *ids {
            if KEPT_COMMANDS.contains(&id) || seen.contains(&id) {
                continue;
            }
            let Some(spec) = pdfcraft_engine::commands::command(id) else { continue };
            seen.push(id);
            out.push(Thing { key: format!("menu:{id}"), kind: Kind::Menu, label: format!("{path} ▸ {}", spec.label) });
        }
    }
    out
}

// ---------- the window ----------

/// The window's tabs.
const TABS: [&str; 3] = ["Closet", "Basement", "Everything"];

fn tab_id() -> egui::Id {
    egui::Id::unique("closet-tab")
}

fn search_id() -> egui::Id {
    egui::Id::unique("closet-search")
}

/// Draws the Closet window when it is open. On the first frame, closes what a saved session
/// had open that the sort has put away.
pub fn window(app: &mut PdfCraftApp, ctx: &egui::Context) {
    if !app.closet_started {
        app.closet_started = true;
        app.closet_tidy();
    }
    if !app.closet_open {
        return;
    }
    let t = Tokens::get(ctx);
    let mut tab: usize = ctx.data(|d| d.get_temp(tab_id()).unwrap_or(0));
    let mut query: String = ctx.data(|d| d.get_temp(search_id()).unwrap_or_default());
    let mut open = true;
    let mut moves: Vec<(String, Shelf)> = Vec::new();
    let mut reset: Option<bool> = None;
    let things = things();
    let shelves: Vec<Shelf> = things.iter().map(|th| app.shelf(&th.key)).collect();
    egui::Window::new(tl!("Closet"))
        .id(egui::Id::unique("closet-window"))
        .open(&mut open)
        .default_pos(egui::pos2(ctx.content_rect().max.x - 460.0, 90.0))
        .default_width(420.0)
        .resizable(true)
        .collapsible(false)
        .frame(egui::Frame::window(&ctx.global_style()).fill(t.panel))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, name) in TABS.iter().enumerate() {
                    if ui.selectable_label(tab == i, RichText::new(tl!(name)).size(13.0)).clicked() {
                        tab = i;
                    }
                }
            });
            let blurb = match tab {
                0 => tl!("Put away, one click from coming back."),
                1 => tl!("Things you'll probably never need. Nothing here is deleted."),
                _ => tl!("Everything that can be put away. Pick where each one lives."),
            };
            ui.label(RichText::new(blurb).size(12.0).color(t.text_muted));
            ui.add_space(4.0);
            ui.add(crate::widgets::line(&mut query).hint_text(tl!("Search")).desired_width(f32::INFINITY));
            ui.add_space(4.0);
            let q = query.trim().to_lowercase();
            let wanted = |s: Shelf| match tab {
                0 => s == Shelf::Closet,
                1 => s == Shelf::Basement,
                _ => true,
            };
            let rows: Vec<(&Thing, Shelf)> = things
                .iter()
                .zip(shelves.iter().copied())
                .filter(|(th, s)| wanted(*s) && (q.is_empty() || th.label.to_lowercase().contains(&q) || tl!(&th.label).to_lowercase().contains(&q)))
                .collect();
            egui::ScrollArea::vertical().id_salt("closet-rows").auto_shrink([false, true]).max_height(460.0).show(ui, |ui| {
                if rows.is_empty() {
                    ui.add_space(8.0);
                    let empty = match (tab, q.is_empty()) {
                        (0, true) => tl!("The Closet is empty."),
                        (1, true) => tl!("The Basement is empty."),
                        _ => tl!("Nothing matches."),
                    };
                    ui.label(RichText::new(empty).color(t.text_muted));
                }
                let mut kind = None;
                for (th, now) in &rows {
                    if kind != Some(th.kind) {
                        kind = Some(th.kind);
                        ui.add_space(6.0);
                        ui.label(RichText::new(tl!(th.kind.heading())).size(11.5).strong().color(t.text_muted));
                    }
                    ui.horizontal(|ui| {
                        ui.set_min_height(24.0);
                        let buttons: [(&'static str, Shelf); 2] = match now {
                            Shelf::Closet => [("To Basement", Shelf::Basement), ("Restore", Shelf::InUse)],
                            Shelf::Basement => [("To Closet", Shelf::Closet), ("Restore", Shelf::InUse)],
                            Shelf::InUse => [("To Basement", Shelf::Basement), ("To Closet", Shelf::Closet)],
                        };
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Right to left: the far shelf first.
                            for (label, to) in buttons {
                                if ui.small_button(tl!(label)).clicked() {
                                    moves.push((th.key.clone(), to));
                                }
                            }
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.add(egui::Label::new(RichText::new(tl!(&th.label)).color(t.text)).truncate());
                            });
                        });
                    });
                }
            });
            ui.add_space(6.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                if crate::widgets::pill_button(ui, tl!("Restore My Starting Sort"), false).clicked() {
                    reset = Some(false);
                }
                if crate::widgets::pill_button(ui, tl!("Bring Everything Back"), false).clicked() {
                    reset = Some(true);
                }
            });
        });
    ctx.data_mut(|d| {
        d.insert_temp(tab_id(), tab);
        d.insert_temp(search_id(), query);
    });
    for (key, to) in moves {
        app.closet_set(&key, to);
    }
    if let Some(all) = reset {
        app.closet_reset(all);
    } else {
        app.closet_tidy();
    }
    if !open {
        app.closet_open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted() -> PdfCraftApp {
        let mut app = PdfCraftApp::new();
        app.closet_sorted = true;
        app
    }

    #[test]
    fn a_new_app_without_the_sort_puts_nothing_away() {
        let app = PdfCraftApp::new();
        assert!(things().iter().all(|t| !app.put_away(&t.key)));
    }

    #[test]
    fn every_shipped_key_names_something_that_can_be_put_away() {
        let keys: Vec<String> = things().into_iter().map(|t| t.key).collect();
        for (key, _) in SHIPPED {
            assert!(keys.iter().any(|k| k == key), "{key} is in the starting sort but can't be put away");
        }
    }

    #[test]
    fn emmas_tools_stay_and_the_rest_go() {
        let app = sorted();
        for id in ["organize", "combine", "comment", "fill_sign", "form", "redact"] {
            assert!(!app.group_put_away(id), "{id} should stay");
        }
        assert_eq!(app.shelf("group:export"), Shelf::Closet);
        assert_eq!(app.shelf("group:scan"), Shelf::Basement);
        for tool in [comments::CommentTool::Note, comments::CommentTool::Highlight, comments::CommentTool::Ink, comments::CommentTool::Eraser] {
            assert!(!app.tool_put_away(tool.command()));
        }
        assert!(app.tool_put_away("comment.underline"));
        assert!(!app.panel_put_away(RightPanel::Comments));
        assert!(!app.panel_put_away(RightPanel::Pages));
        assert!(app.panel_put_away(RightPanel::Layers));
    }

    #[test]
    fn menu_items_follow_their_tool_sets_unless_moved() {
        let mut app = sorted();
        // Export a PDF is in the Closet, so its menu items are too.
        assert_eq!(app.shelf("menu:export.docx"), Shelf::Closet);
        // Delete pages belongs to Organize pages too, which stays.
        assert!(!app.put_away("menu:page.delete"));
        // Restoring the item alone brings it back while its tool set stays away.
        app.closet_set("menu:export.docx", Shelf::InUse);
        assert!(!app.put_away("menu:export.docx"));
        // Restoring the tool set brings its other items back.
        app.closet_set("group:export", Shelf::InUse);
        assert!(!app.put_away("menu:export.html"));
        assert!(app.closet.contains_key("group:export"));
    }

    #[test]
    fn the_menus_lose_what_is_put_away() {
        let app = sorted();
        let ctx = egui::Context::default();
        let mut bar = app.menu_bar(&ctx);
        app.prune_menus(&mut bar);
        let mut ids = Vec::new();
        fn walk(nodes: &[craft_menu::Node], ids: &mut Vec<String>) {
            for n in nodes {
                match n {
                    craft_menu::Node::Item(i) => ids.push(i.command.id.clone()),
                    craft_menu::Node::Submenu { children, .. } => walk(children, ids),
                    _ => {}
                }
            }
        }
        for m in &bar.menus {
            walk(&m.children, &mut ids);
        }
        assert!(!ids.iter().any(|i| i == "export.docx"));
        assert!(!ids.iter().any(|i| i == "ocr.recognize"));
        assert!(ids.iter().any(|i| i == "page.combine"));
        assert!(ids.iter().any(|i| i == "print.dialog"));
        assert!(ids.iter().any(|i| i == "window.closet"));
    }

    #[test]
    fn reset_brings_everything_back_or_restores_the_sort() {
        let mut app = sorted();
        app.closet_reset(true);
        assert!(things().iter().all(|t| !app.put_away(&t.key)));
        app.closet_reset(false);
        assert!(app.group_put_away("scan"));
    }

    #[test]
    fn closet_commands_check_their_input() {
        let mut app = sorted();
        assert!(app.closet_command("closet.set", &json!({ "key": "group:nope", "shelf": "closet" })).is_some_and(|r| r.is_err()));
        assert!(app.closet_command("closet.set", &json!({ "key": "group:scan", "shelf": "attic" })).is_some_and(|r| r.is_err()));
        assert!(app.closet_command("closet.set", &json!({ "key": "group:scan", "shelf": "inUse" })).is_some_and(|r| r.is_ok()));
        assert!(!app.group_put_away("scan"));
        assert!(app.closet_command("other", &Value::Null).is_none());
    }

    #[test]
    fn putting_away_the_open_panel_closes_it() {
        let mut app = sorted();
        app.right = Some(RightPanel::Comments);
        app.closet_set("panel:comments", Shelf::Closet);
        app.closet_tidy();
        assert_eq!(app.right, None);
    }
}
