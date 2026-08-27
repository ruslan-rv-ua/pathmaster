//! View → "PATH Tree…": the modal, per-Scope comprehension surface
//! (v0.2.0 §6, `CONTEXT.md` **Tree View**).
//!
//! What the dialog shows is a [`Tree`] the caller built and handed over — the
//! active Scope's Filtered View snapshotted at open. Nothing here reads a
//! Working Copy, a Session or the narrowing criteria, and **nothing here is
//! live**: no diagnostics, no refresh affordance, no Timer of its own, which
//! is what keeps a modal's nested event loop free of the borrow hazard every
//! other dialog in this application is careful about (ADR-0011). Reopening is
//! the refresh.
//!
//! **The widget is the native one.** `TreeCtrl` on MSW is `SysTreeView32`, so
//! levels, expanded/collapsed state and the item text all reach NVDA for free
//! — which is why a leaf's label carries everything it has to say (a tree item
//! has no columns and no description, and tooltips NVDA demonstrably cannot
//! reach). The labels are the Catalogue's, composed once per node as the tree
//! is filled.
//!
//! **The activation handler is the single home of the commit logic.** wxMSW
//! refuses to preprocess an unmodified Enter for a tree (`MSWShouldPreProcess­-`
//! `Message`, pinned 3.3.3), so Enter never reaches the dialog's default
//! button and always arrives here as `ITEM_ACTIVATED` — one gesture, one
//! handler, and the "Go to entry" button calls into the same answer rather
//! than repeating it. A leaf commits; anything else toggles, because that is
//! the node's own default action and native `SysTreeView32` does nothing at
//! all with Enter.
//!
//! **Identity, never text.** A `TreeItemId` is a borrowed handle with no
//! equality and no lifetime worth keeping, so what a selection is turned into
//! is its **position path** — its index among its siblings, level by level —
//! and [`Tree::at`] walks the same path back to the node. The tree cannot
//! change under the walk: it is a snapshot, and this dialog never rebuilds it.

use std::rc::Rc;

use pathmaster_core::catalogue::Catalogue;
use pathmaster_core::msgids;
use pathmaster_core::session::EntryId;
use pathmaster_core::tree::{Node, Tree};
use wxdragon::prelude::*;

use crate::catalog::translate;
use crate::ui::door;

/// The two ways out. Local to this module, like every other dialog's:
/// `door::show` hands one back and nothing else binds them.
const ID_COMMIT: Id = ID_HIGHEST + 131;
const ID_ABANDON: Id = ID_HIGHEST + 132;

/// The tree's size in DIP. Its own fit would size it to the labels it happens
/// to hold, which for a compressed chain is a whole path and for a bare drive
/// root is two characters. Both dimensions are given because a tree is browsed
/// vertically and a dialog that grew a scrollbar on its third row would be one
/// more thing to walk past. It crosses the FFI boundary, where wxdragon
/// applies `FromDIP` for us.
const TREE_WIDTH_DIP: i32 = 560;
const TREE_HEIGHT_DIP: i32 = 380;

/// Opens the Tree View over `tree` and answers with the Entry the user chose
/// to go to, or `None` if they left without choosing one.
///
/// `title` is the Catalogue's Scope-named title — the whole of what NVDA
/// speaks when the modal opens, and so the only place the dialog says which
/// PATH it is showing.
///
/// The answer is an [`EntryId`] and never a path: two Entries can read
/// identically and a leaf must lead to exactly one row (v0.2.0 §6).
pub fn ask_for_entry_to_go_to(
    parent: &dyn WxWidget,
    catalogue: &Catalogue,
    title: &str,
    tree: Tree,
) -> Option<EntryId> {
    let dialog = Dialog::builder(parent, title).build();
    let panel = Panel::builder(&dialog).build();

    // Created before the buttons because creation order is the Tab order, and
    // §6 fixes it: tree → Go to entry → Cancel.
    //
    // `HideRoot` is how "no artificial super-root" is spelled to wx: the tree
    // needs a root to hang items from, and hiding it makes the drive roots and
    // the groups the top level they are — in the accessibility tree as much as
    // on screen. `Single` is the app's real shape here as it is in the lists:
    // one leaf leads to one row. Editing labels is not among the styles, for
    // the reason it is absent from the lists too — this surface reads.
    //
    // `Default` is `HasButtons | LinesAtRoot`, and the pair is **measured**,
    // not decoration: comctl32 draws no expand button on a root item unless
    // `TVS_LINESATROOT` is set beside `TVS_HASBUTTONS`, so with buttons alone
    // the whole top level renders as a flat list of drive roots that cannot be
    // opened with the mouse at all.
    let widget = TreeCtrl::builder(&panel)
        .with_style(TreeCtrlStyle::Default | TreeCtrlStyle::Single | TreeCtrlStyle::HideRoot)
        .with_size(Size::new(TREE_WIDTH_DIP, TREE_HEIGHT_DIP))
        .build();

    let commit = Button::builder(&panel)
        .with_id(ID_COMMIT)
        .with_label(&translate(msgids::BUTTON_GO_TO_ENTRY))
        .build();
    let abandon = Button::builder(&panel)
        .with_id(ID_ABANDON)
        .with_label(&translate(msgids::BUTTON_DIALOG_CANCEL))
        .build();
    // Default even while disabled: it is the Enter gesture's visible form, and
    // a dialog whose default moved with the selection would be one whose Enter
    // means different things in different places (§6, the UX guide's rule that
    // the double-click action has a redundant visible form).
    commit.set_default();

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    buttons.add_stretch_spacer(1);
    buttons.add(&commit, 0, SizerFlag::All, 4);
    buttons.add(&abandon, 0, SizerFlag::All, 4);

    let inner = BoxSizer::builder(Orientation::Vertical).build();
    inner.add(&widget, 1, SizerFlag::Expand | SizerFlag::All, 8);
    inner.add_sizer(&buttons, 0, SizerFlag::Expand | SizerFlag::All, 8);
    panel.set_sizer(inner, true);

    let outer = BoxSizer::builder(Orientation::Vertical).build();
    outer.add(&panel, 1, SizerFlag::Expand, 0);
    dialog.set_sizer_and_fit(outer, true);
    dialog.centre();

    let first = fill(&widget, catalogue, &tree);

    // Shared because three handlers read it and the dialog outlives none of
    // them: the walk from a selection back to a node is the only thing any of
    // them does with it, and they all have to walk the same tree.
    let tree = Rc::new(tree);
    let held = Rc::clone(&tree);
    widget.on_selection_changed(move |_| {
        // The visible, NVDA-readable statement of the leaf-only commit rule:
        // an inner node or a group has no Entry, so there is nowhere to go.
        commit.enable(chosen(&widget, &held).is_some());
    });
    let held = Rc::clone(&tree);
    widget.on_item_activated(move |event| {
        // **Consumed, always** — and this is measured, not tidiness. wxdragon
        // re-skips every event before it calls a handler, so a handler that
        // says nothing leaves it skipped; wxMSW then reads that as "the user
        // did not handle the activation" and lets the *tree* act on the
        // double-click too (`*result = processed`, `msw/treectrl.cpp`). The
        // toggle below and comctl32's own would then both fire, and a
        // double-click on a folder would visibly do nothing at all.
        event.event.skip(false);
        match widget.get_selection() {
            // A leaf: the commit, ending the dialog on the selection the
            // caller reads back below.
            Some(item) if node_at(&widget, &held, &item).is_some_and(has_entry) => {
                dialog.end_modal(ID_COMMIT);
            }
            // An inner node or a group: its own default action, performed
            // here because nothing else will — native `SysTreeView32` does
            // nothing with Enter, and the double-click's own toggle is what
            // the line above has just taken responsibility for.
            Some(item) => widget.toggle(&item),
            None => {}
        }
    });
    // The button is the same gesture by another route, so it is the same
    // answer: it never commits what Enter would not.
    commit.on_click(move |_| dialog.end_modal(ID_COMMIT));
    abandon.on_click(move |_| dialog.end_modal(ID_ABANDON));
    dialog.set_escape_id(ID_ABANDON);

    // Initial focus on the first top-level node — and the button pointed at
    // whatever that turned out to be, asked directly rather than left to the
    // selection event `select_item` raises: a first frame that depended on
    // that event would be one rule written in two places.
    if let Some(item) = &first {
        widget.select_item(item);
        widget.ensure_visible(item);
    }
    commit.enable(chosen(&widget, &tree).is_some());
    widget.set_focus();

    let committed = door::show(&dialog) == ID_COMMIT;
    // Read before the window goes: a destroyed control answers with nothing.
    // The selection is what both routes commit — wx selects an item before it
    // activates one, whether the gesture was Enter or a double-click — so
    // there is one answer here and no second copy of it kept by a handler.
    let entry = committed.then(|| chosen(&widget, &tree)).flatten();
    dialog.destroy();
    entry
}

/// Builds every item, depth-first in the tree's own order, and hands back the
/// first top-level one — where focus opens (§6).
///
/// **All of it, or none of it.** A widget item's only identity here is its
/// place among its siblings, so a level built with one item missing is a tree
/// whose positions name different Entries than the snapshot's do — and the
/// jump would land on a row the user did not choose, which is worse than not
/// landing at all. A tree that cannot be built whole is therefore emptied: the
/// dialog then says plainly, with a disabled button, that there is nowhere to
/// go.
fn fill(widget: &TreeCtrl, catalogue: &Catalogue, tree: &Tree) -> Option<TreeItemId> {
    // Hidden by the style: wx needs a root to append to, and this is not a
    // node — it has no label, and nothing ever walks to it.
    let root = widget.add_root("", None, None)?;
    append(widget, catalogue, &root, tree.roots()).unwrap_or_else(|Incomplete| {
        widget.delete_all_items();
        None
    })
}

/// The one thing appending can fail at: wx refused an item, which it does only
/// for a control that is already gone.
struct Incomplete;

/// Appends one level and everything under it, and answers with the first item
/// of that level — kept as it is made rather than asked for afterwards,
/// because the item this level hangs from may be the hidden root, which is not
/// a native item at all.
fn append(
    widget: &TreeCtrl,
    catalogue: &Catalogue,
    parent: &TreeItemId,
    nodes: &[Node],
) -> Result<Option<TreeItemId>, Incomplete> {
    let mut first = None;
    for node in nodes {
        let label = catalogue.tree_label(node);
        let item = widget
            .append_item(parent, &label, None, None)
            .ok_or(Incomplete)?;
        first.get_or_insert_with(|| item.clone());
        append(widget, catalogue, &item, node.children())?;
    }
    Ok(first)
}

/// The Entry the selection stands for, or `None` while it is an inner node, a
/// group, or nothing at all — which is exactly the "Go to entry" rule.
fn chosen(widget: &TreeCtrl, tree: &Tree) -> Option<EntryId> {
    let item = widget.get_selection()?;
    node_at(widget, tree, &item)?.entry().map(|leaf| leaf.id)
}

fn has_entry(node: &Node) -> bool {
    node.entry().is_some()
}

/// The node a tree item stands for, found by walking the item's position path
/// back down the snapshot it was built from.
fn node_at<'a>(widget: &TreeCtrl, tree: &'a Tree, item: &TreeItemId) -> Option<&'a Node> {
    tree.at(&position_path(widget, item))
}

/// One item's place in the tree: its index among its siblings at each level,
/// outermost first.
///
/// Counted by walking back through the previous siblings rather than forward
/// through the parent's children, because that is the walk that terminates on
/// the item itself — a `TreeItemId` cannot be compared with another, so a
/// forward walk would have no way to recognise its own target. The hidden root
/// is where it stops: wx answers "no parent" for it and for nothing else.
fn position_path(widget: &TreeCtrl, item: &TreeItemId) -> Vec<usize> {
    let mut path = Vec::new();
    let mut current = item.clone();
    while let Some(parent) = widget.get_item_parent(&current) {
        let mut index = 0;
        let mut back = current.clone();
        while let Some(previous) = widget.get_prev_sibling(&back) {
            index += 1;
            back = previous;
        }
        path.push(index);
        current = parent;
    }
    path.reverse();
    path
}
