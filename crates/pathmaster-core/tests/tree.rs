//! The Tree View's merge algorithm at the crate boundary (v0.2.0 spec §6,
//! ticket impl-07).
//!
//! One question: given the Entries a Scope's Filtered View is showing, what
//! shape does the modal put them in? The answer is a prefix tree over the
//! **expanded reading** — Normalisation's own, undefined `%VAR%` left literal,
//! and taking no Expansion Mode at all, which is why no test here can pass one
//! in. Everything about the shape is decided here; everything about the words
//! is the Catalogue's, and the label tests at the end read it through the same
//! untranslated lookup the composition tests use.

use pathmaster_core::catalogue::{Catalogue, Lookup};
use pathmaster_core::diagnostics::Issue;
use pathmaster_core::normalize::Environment;
use pathmaster_core::session::{Entry, EntryId, Scope, ScopeValue, Session, ValueType};
use pathmaster_core::tree::{Group, Node, Tree};

/// A fixed environment, looked up case-insensitively as Windows' own is.
struct Env(&'static [(&'static str, &'static str)]);

impl Environment for Env {
    fn lookup(&self, name: &str) -> Option<String> {
        self.0
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| (*value).to_string())
    }
}

const ENV: Env = Env(&[("SystemRoot", r"C:\Windows"), ("JAVA_HOME", r"C:\jdk21")]);

/// The msgid-as-translation lookup: the shape tests never reach it, and the
/// label tests read the English the registry is written in.
struct Untranslated;

impl Lookup for Untranslated {
    fn translate(&self, msgid: &str) -> String {
        msgid.to_string()
    }

    fn translate_plural(&self, singular: &str, plural: &str, n: u32) -> String {
        if n == 1 {
            singular.to_string()
        } else {
            plural.to_string()
        }
    }
}

/// A Session over `raws`, which is the only way to come by Entry ids — the
/// tree carries them and nothing else identifies a row.
fn session(raws: &[&str]) -> Session {
    Session::new(
        Scope::User,
        ScopeValue::Present {
            value_type: ValueType::RegExpandSz,
            raw: raws.join(";"),
        },
        true,
    )
}

/// The tree over every Entry of `session`, none of them flagged.
fn tree(session: &Session) -> Tree {
    flagged(session, &[])
}

/// The tree over every Entry of `session`, with `issues` on the Entry at each
/// listed position.
fn flagged(session: &Session, issues: &[(usize, &[Issue])]) -> Tree {
    let entries: Vec<(EntryId, &str, &[Issue])> = session
        .entries()
        .iter()
        .enumerate()
        .map(|(index, entry): (usize, &Entry)| {
            let found = issues
                .iter()
                .find(|(at, _)| *at == index)
                .map_or(&[][..], |(_, issues)| *issues);
            (entry.id(), entry.raw(), found)
        })
        .collect();
    Tree::of(entries, &ENV)
}

/// Every node's label, depth-first, indented one space per level — the whole
/// shape as one comparable string.
fn shape(tree: &Tree) -> String {
    let catalogue = Catalogue::new(Untranslated);
    let mut out = String::new();
    fn walk(catalogue: &Catalogue, nodes: &[Node], depth: usize, out: &mut String) {
        for node in nodes {
            out.push_str(&" ".repeat(depth));
            out.push_str(&catalogue.tree_label(node));
            out.push('\n');
            walk(catalogue, node.children(), depth + 1, out);
        }
    }
    walk(&catalogue, tree.roots(), 0, &mut out);
    out
}

/// The label of the node `path` names.
fn label_at(tree: &Tree, path: &[usize]) -> String {
    Catalogue::new(Untranslated).tree_label(tree.at(path).expect("a node at that path"))
}

// ------------------------------------------------------------- one Entry

#[test]
fn one_entry_is_one_leaf_and_there_is_no_super_root() {
    // No artificial "PATH" root: a single Entry is a single top-level node,
    // and its whole chain compresses into that one label (v0.2.0 §6).
    let session = session(&[r"C:\tools\bin"]);
    let tree = tree(&session);

    assert_eq!(shape(&tree), "C:\\tools\\bin\n");
    assert_eq!(tree.roots().len(), 1);
    assert_eq!(
        tree.roots()[0].entry().map(|leaf| leaf.id),
        Some(session.entries()[0].id())
    );
}

#[test]
fn an_empty_view_is_an_empty_tree() {
    let session = session(&[]);

    assert!(tree(&session).roots().is_empty());
}

// ---------------------------------------------------------- the prefix tree

#[test]
fn entries_split_at_the_first_fork_and_the_rest_compresses() {
    // The shape the feature exists for: one node per shared prefix, and the
    // unshared tails joined into one label each (v0.2.0 §6).
    let session = session(&[
        r"C:\Program Files\Java\jdk-21\bin",
        r"C:\Program Files\Git\cmd",
        r"C:\Windows\system32",
    ]);

    assert_eq!(
        shape(&tree(&session)),
        "C:\n \
         Program Files\n  \
         Git\\cmd\n  \
         Java\\jdk-21\\bin\n \
         Windows\\system32\n"
    );
}

#[test]
fn siblings_sort_alphabetically_and_case_insensitively() {
    let session = session(&[r"C:\zebra", r"C:\Apple", r"C:\mango"]);

    assert_eq!(shape(&tree(&session)), "C:\n Apple\n mango\n zebra\n");
}

#[test]
fn segments_differing_only_in_case_are_one_node() {
    // Merging is Normalisation's question — "are these the same path?" — so a
    // prefix spelled two ways is one directory and one node.
    let session = session(&[r"C:\Tools\bin", r"C:\tools\lib"]);

    assert_eq!(shape(&tree(&session)), "C:\\Tools\n bin\n lib\n");
}

#[test]
fn slash_direction_is_reconciled_before_the_split() {
    let session = session(&[r"C:/tools/bin", r"C:\tools\lib"]);

    assert_eq!(shape(&tree(&session)), "C:\\tools\n bin\n lib\n");
}

#[test]
fn a_trailing_separator_adds_no_empty_segment() {
    let session = session(&[r"C:\tools\", r"C:\tools\bin"]);

    assert_eq!(shape(&tree(&session)), "C:\n tools\n tools\\bin\n");
}

#[test]
fn a_bare_drive_root_is_a_top_level_leaf() {
    let session = session(&[r"C:\"]);

    assert_eq!(shape(&tree(&session)), "C:\n");
    assert!(tree(&session).roots()[0].entry().is_some());
}

#[test]
fn a_unc_path_roots_at_its_share() {
    // The share is the root a UNC path hangs from — `\\server` and `share`
    // are not two levels of anything the user can browse.
    let session = session(&[r"\\build\tools\bin", r"\\build\tools\lib"]);

    assert_eq!(shape(&tree(&session)), "\\\\build\\tools\n bin\n lib\n");
}

#[test]
fn quotes_are_read_past_so_a_quoted_entry_keeps_its_place() {
    // Normalisation's reading strips one surrounding pair, so a quoted Entry
    // sits under its drive rather than under a root spelled `"C:`.
    let session = session(&[r#""C:\tools\bin""#, r"C:\tools\lib"]);

    assert_eq!(
        shape(&tree(&session)),
        "C:\\tools\n bin (\"C:\\tools\\bin\")\n lib\n"
    );
}

// ------------------------------------------------------- one leaf per Entry

#[test]
fn duplicates_are_sibling_leaves() {
    // A leaf must lead to exactly one row, so two Entries naming one path are
    // two leaves — never one node standing for both (v0.2.0 §6).
    let session = session(&[r"C:\tools\bin", r"C:\tools\bin"]);
    let tree = tree(&session);

    assert_eq!(shape(&tree), "C:\\tools\n bin\n bin\n");
    assert_eq!(
        tree.at(&[0, 0]).and_then(Node::entry).map(|leaf| leaf.id),
        Some(session.entries()[0].id())
    );
    assert_eq!(
        tree.at(&[0, 1]).and_then(Node::entry).map(|leaf| leaf.id),
        Some(session.entries()[1].id())
    );
}

#[test]
fn an_entry_that_is_also_another_entrys_prefix_keeps_its_own_leaf() {
    // `C:\tools` is an Entry and `C:\tools\bin`'s prefix. The Entry's leaf is
    // never swallowed by the prefix node: every Entry is reachable.
    let session = session(&[r"C:\tools", r"C:\tools\bin", r"C:\tools\lib"]);
    let tree = tree(&session);

    assert_eq!(shape(&tree), "C:\n tools\n tools\n  bin\n  lib\n");
    assert_eq!(
        tree.at(&[0, 0]).and_then(Node::entry).map(|leaf| leaf.id),
        Some(session.entries()[0].id())
    );
    assert!(tree.at(&[0, 1]).expect("the prefix node").entry().is_none());
}

// ------------------------------------------------------------- the groups

#[test]
fn an_undefined_variable_is_a_literal_leaf_in_its_own_group() {
    let session = session(&[r"C:\tools", r"%NOPE%\bin"]);

    assert_eq!(
        shape(&tree(&session)),
        "C:\\tools\nUnresolved variables\n %NOPE%\\bin\n"
    );
}

#[test]
fn a_defined_variable_is_placed_by_what_it_expands_to() {
    // The base is always the expanded reading — the same one diagnostics uses
    // — so `%JAVA_HOME%\bin` is a directory under `C:`, not a group member.
    let session = session(&[r"%JAVA_HOME%\bin", r"C:\jdk21\lib"]);

    assert_eq!(
        shape(&tree(&session)),
        "C:\\jdk21\n bin (%JAVA_HOME%\\bin)\n lib\n"
    );
}

#[test]
fn a_variable_further_along_leaves_the_shape_legible() {
    // Only a reference in the *leading* position makes the path's position
    // unanswerable; one further along is an ordinary segment.
    let session = session(&[r"C:\tools\%NOPE%"]);

    assert_eq!(shape(&tree(&session)), "C:\\tools\\%NOPE%\n");
}

#[test]
fn a_relative_entry_is_a_literal_leaf_in_its_own_group() {
    let session = session(&[r"C:\tools", r"..\bin", "node_modules"]);

    assert_eq!(
        shape(&tree(&session)),
        "C:\\tools\nRelative entries\n ..\\bin\n node_modules\n"
    );
}

#[test]
fn an_empty_entry_has_no_filesystem_position_either() {
    // Two groups, and an Entry with no usable path text belongs to neither
    // drive nor variable — it is grouped rather than dropped, because no
    // Entry may be missing from the tree.
    let session = session(&[r"C:\tools", "   "]);

    assert_eq!(
        shape(&tree(&session)),
        "C:\\tools\nRelative entries\n    \n"
    );
}

#[test]
fn the_groups_sort_after_the_drive_roots_unresolved_first() {
    let session = session(&[r"..\bin", r"%NOPE%\bin", r"D:\tools", r"C:\tools"]);

    assert_eq!(
        shape(&tree(&session)),
        "C:\\tools\nD:\\tools\nUnresolved variables\n %NOPE%\\bin\nRelative entries\n ..\\bin\n"
    );
}

#[test]
fn an_empty_group_is_not_shown() {
    let session = session(&[r"C:\tools"]);

    assert_eq!(tree(&session).roots().len(), 1);
}

#[test]
fn a_groups_members_sort_alphabetically_too() {
    let session = session(&[r"..\zebra", r"..\Apple"]);

    assert_eq!(
        shape(&tree(&session)),
        "Relative entries\n ..\\Apple\n ..\\zebra\n"
    );
}

#[test]
fn a_group_never_compresses_into_its_one_member() {
    // Compression is what a *path* chain does; a group's name is the whole
    // reason it exists, and joining it onto its member would lose it.
    let session = session(&[r"%NOPE%\bin"]);

    assert_eq!(
        shape(&tree(&session)),
        "Unresolved variables\n %NOPE%\\bin\n"
    );
}

// -------------------------------------------------------- the position path

#[test]
fn a_position_path_names_the_node_it_walks_to() {
    let session = session(&[
        r"C:\Program Files\Java\jdk-21\bin",
        r"C:\Program Files\Git\cmd",
    ]);
    let tree = tree(&session);

    assert_eq!(label_at(&tree, &[0]), "C:\\Program Files");
    assert_eq!(label_at(&tree, &[0, 0]), "Git\\cmd");
    assert_eq!(label_at(&tree, &[0, 1]), "Java\\jdk-21\\bin");
}

#[test]
fn a_position_path_past_the_end_names_nothing() {
    let session = session(&[r"C:\tools\bin"]);
    let tree = tree(&session);

    assert!(tree.at(&[1]).is_none());
    assert!(tree.at(&[0, 0]).is_none());
}

#[test]
fn the_empty_position_path_names_nothing_because_there_is_no_root() {
    let session = session(&[r"C:\tools\bin"]);

    assert!(tree(&session).at(&[]).is_none());
}

// ------------------------------------------------------------- the labels

#[test]
fn a_leaf_speaks_its_chain_alone_when_the_raw_reads_the_same() {
    let session = session(&[r"C:\tools\bin", r"C:\tools\lib"]);
    let tree = tree(&session);

    assert_eq!(label_at(&tree, &[0, 0]), "bin");
}

#[test]
fn a_leaf_carries_the_raw_form_only_when_it_differs() {
    let session = session(&[r"%JAVA_HOME%\bin", r"C:\jdk21\lib"]);
    let tree = tree(&session);

    assert_eq!(label_at(&tree, &[0, 0]), "bin (%JAVA_HOME%\\bin)");
    assert_eq!(label_at(&tree, &[0, 1]), "lib");
}

#[test]
fn a_leaf_carries_the_status_columns_own_words() {
    // The exact Status-column string, comma-joined most-severe-first — one
    // Issue has one name, wherever it is read (ADR-0004).
    let session = session(&[r"%JAVA_HOME%\bin", r"C:\jdk21\lib"]);
    let catalogue = Catalogue::new(Untranslated);
    let tree = flagged(&session, &[(0, &[Issue::Missing, Issue::Duplicate])]);

    assert_eq!(
        catalogue.tree_label(tree.at(&[0, 0]).expect("the flagged leaf")),
        "bin (%JAVA_HOME%\\bin) — Missing, Duplicate"
    );
    assert_eq!(
        catalogue.status_column(&[Issue::Missing, Issue::Duplicate]),
        "Missing, Duplicate"
    );
}

#[test]
fn a_leaf_with_the_raw_matching_still_carries_its_issue_suffix() {
    let session = session(&[r"C:\tools\bin", r"C:\tools\lib"]);
    let tree = flagged(&session, &[(0, &[Issue::Missing])]);

    assert_eq!(label_at(&tree, &[0, 0]), "bin — Missing");
}

#[test]
fn an_inner_node_carries_no_suffix_of_its_own() {
    // Status belongs to the Entry, not to the prefix: the parent of a flagged
    // leaf says nothing about it.
    let session = session(&[r"C:\tools\bin", r"C:\tools\lib"]);
    let tree = flagged(&session, &[(0, &[Issue::Missing])]);

    assert_eq!(label_at(&tree, &[0]), "C:\\tools");
}

#[test]
fn a_group_is_named_by_the_catalogue_and_carries_no_suffix() {
    let session = session(&[r"%NOPE%\bin"]);
    let tree = flagged(&session, &[(0, &[Issue::Missing])]);

    assert_eq!(label_at(&tree, &[0]), Group::Unresolved.catalogue_msgid());
    assert_eq!(label_at(&tree, &[0, 0]), "%NOPE%\\bin — Missing");
}

#[test]
fn both_group_names_are_registered_catalogue_strings() {
    for group in [Group::Unresolved, Group::Relative] {
        let msgid = group.catalogue_msgid();
        assert!(
            pathmaster_core::msgids::REGISTRY
                .iter()
                .any(|entry| entry.msgid == msgid),
            "the tree shows {msgid:?} but the Catalogue does not hold it"
        );
    }
}
