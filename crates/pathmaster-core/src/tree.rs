//! The Tree View's shape: a Scope's Entries merged into a prefix tree
//! (v0.2.0 spec §6, `CONTEXT.md`).
//!
//! A **Tree View** is a modal, per-Scope comprehension surface — the Scope's
//! Filtered View snapshotted when the dialog opens and shaped as the
//! filesystem. What lives here is the shaping and nothing else: the dialog's
//! own behaviour is the window's, and the words every node speaks are the
//! Catalogue's ([`Catalogue::tree_label`]). This module is pure, takes no
//! Expansion Mode, and cannot be given one — which is the whole of "the base
//! is always the expanded reading, independent of Expansion Mode".
//!
//! The rules, each of which a test here fixes:
//!
//! * **Merged by the expanded reading** — Normalisation's own, so a `%VAR%`
//!   this run defines is placed by what it resolves to and one it does not is
//!   left literal. Quotes are read past for the same reason diagnostics reads
//!   past them: the quoted spelling still names a directory, and a leading `"`
//!   would otherwise invent a drive root nobody has.
//! * **One leaf per Entry.** A leaf leads to exactly one row, so two Entries
//!   naming one path are two sibling leaves, and an Entry that happens to be
//!   another's prefix keeps a leaf of its own beside the prefix node. Only the
//!   segments *above* an Entry's last are ever merged.
//! * **Single-child chains compress** into one node with the joined label,
//!   which is what keeps a real `PATH` inside a browsable depth.
//! * **Siblings sort alphabetically, case-insensitive** — by the same fold
//!   that decides what merges, so two siblings that would merge sort together.
//!   PATH order belongs to the main list and cannot survive prefix-merging;
//!   where the fold ties, the Working Copy's order breaks it.
//! * **Entries no drive can hold are grouped, never dropped**: the two
//!   [`Group`]s sit after the drive roots and are absent while empty. There is
//!   **no artificial super-root** — a `PATH` of one Entry is a tree of one
//!   node.
//!
//! [`Catalogue::tree_label`]: crate::catalogue::Catalogue::tree_label

use crate::diagnostics::{is_fully_qualified, Issue};
use crate::msgids;
use crate::normalize::{expand, strip_quotes, Environment, Normalised};
use crate::session::EntryId;

/// A top-level group for the Entries no drive root can hold (v0.2.0 §6).
///
/// Two, and closed: an Entry is grouped because its filesystem position is
/// unanswerable (a `%VAR%` this run does not define stands where the root
/// would) or because it has none (it is not fully qualified). Nothing else can
/// be true of an Entry the prefix tree cannot take, so nothing else is a
/// group — and an Empty Entry, which is no path at all, is grouped with the
/// unqualified rather than dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    /// The Entry begins with a `%NAME%` this run does not define.
    Unresolved,
    /// The Entry is not fully qualified — what it names depends on the
    /// process's current state, so it has no place under any drive.
    Relative,
}

impl Group {
    /// Both groups in the order they stand under the drive roots.
    ///
    /// **Declared rather than sorted**, unlike every other set of siblings:
    /// the names are translated, and an order read off them would put the two
    /// groups one way round in English and the other in Ukrainian. Two nodes
    /// whose position moved with the Interface Language would be a tree the
    /// user has to relearn.
    pub const ORDER: [Group; 2] = [Group::Unresolved, Group::Relative];

    /// The Catalogue string this group is named by (v0.2.0 §14).
    pub fn catalogue_msgid(self) -> &'static str {
        match self {
            Group::Unresolved => msgids::TREE_UNRESOLVED_VARIABLES,
            Group::Relative => msgids::TREE_RELATIVE_ENTRIES,
        }
    }

    /// Where this group's members collect while the tree is being built —
    /// read off [`ORDER`](Self::ORDER) rather than from the discriminant, so
    /// the one list that fixes the order is the one that indexes them.
    fn slot(self) -> usize {
        Group::ORDER
            .iter()
            .position(|group| *group == self)
            .expect("every group is in ORDER")
    }
}

/// The Entry behind one leaf — everything its spoken label is composed from,
/// and the identity the leaf-jump lands by.
///
/// The id is the point: activating a leaf selects **that Entry's** row, by
/// identity and never by text, which is the one thing that survives a
/// duplicate (v0.2.0 §6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leaf {
    pub id: EntryId,
    /// The Entry exactly as stored — what the label shows in parentheses when
    /// it differs from the expansion.
    pub raw: String,
    /// The expanded reading the tree was built over.
    pub expanded: String,
    /// What the last completed pass found about this Entry, most-severe-first
    /// — a snapshot like the rest of the tree, never live.
    pub issues: Vec<Issue>,
}

/// One node of the tree.
///
/// Three kinds and no fourth: a shared prefix, an Entry, or one of the two
/// groups. A leaf never has children — that is what lets "Go to entry" be
/// exactly "the selection is a leaf" — and a group only ever holds leaves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// A prefix shared by everything beneath it: one segment, or the joined
    /// chain a compression left behind.
    Branch { chain: String, children: Vec<Node> },
    /// One Entry, standing at the segment or joined chain that reaches it.
    Leaf { chain: String, entry: Leaf },
    /// One of the two groups, named by the Catalogue rather than by a chain.
    Group { group: Group, children: Vec<Node> },
}

impl Node {
    /// What hangs beneath this node — nothing at all, for a leaf.
    pub fn children(&self) -> &[Node] {
        match self {
            Node::Branch { children, .. } | Node::Group { children, .. } => children,
            Node::Leaf { .. } => &[],
        }
    }

    /// The Entry this node stands for, or `None` for a prefix or a group —
    /// which is the whole of the commit rule: "Go to entry" is available
    /// exactly when this answers (v0.2.0 §6).
    pub fn entry(&self) -> Option<&Leaf> {
        match self {
            Node::Leaf { entry, .. } => Some(entry),
            Node::Branch { .. } | Node::Group { .. } => None,
        }
    }
}

/// One Scope's Entries, shaped.
///
/// Built once and never updated: the dialog it feeds is a snapshot, and
/// reopening is the refresh (v0.2.0 §6).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tree {
    roots: Vec<Node>,
}

impl Tree {
    /// Shapes the Entries a Filtered View is showing, in the order it shows
    /// them.
    ///
    /// Each arrives as the three things a leaf needs: its identity, its raw
    /// text, and what the last completed pass found about it. The environment
    /// is injected like every other reading of it — core takes no OS call.
    pub fn of<'a>(
        entries: impl IntoIterator<Item = (EntryId, &'a str, &'a [Issue])>,
        env: &dyn Environment,
    ) -> Tree {
        let mut drives: Vec<Child> = Vec::new();
        let mut grouped: Vec<Vec<Child>> = Group::ORDER.iter().map(|_| Vec::new()).collect();

        for (id, raw, issues) in entries {
            // Normalisation's own reading, and only its first two steps: the
            // quote-stripped, expanded text is the path this Entry names. The
            // trailing separator and the case fold answer "are these the
            // same?", which is what `fold` below asks of one segment at a time.
            let expansion = expand(strip_quotes(raw), env);
            let leaf = Leaf {
                id,
                raw: raw.to_string(),
                expanded: expansion.text.clone(),
                issues: issues.to_vec(),
            };
            let group = if expansion.starts_unresolved {
                Some(Group::Unresolved)
            } else if !is_fully_qualified(&expansion.text) {
                Some(Group::Relative)
            } else {
                None
            };
            match group {
                // A group holds literal leaves: its members have no filesystem
                // position, so there is no prefix to merge them by.
                Some(group) => grouped[group.slot()].push(Child::leaf(expansion.text, leaf)),
                None => {
                    let segments = segments(&expansion.text);
                    let (chain, above) =
                        segments.split_last().expect("a qualified path has a root");
                    insert(&mut drives, above, Child::leaf(chain.clone(), leaf));
                }
            }
        }

        let mut roots: Vec<Node> = finish(drives);
        for (group, members) in Group::ORDER.into_iter().zip(grouped) {
            // Absent while empty: a group that names nothing is a node the
            // user opens only to find it was not about them.
            if !members.is_empty() {
                roots.push(Node::Group {
                    group,
                    children: finish(members),
                });
            }
        }
        Tree { roots }
    }

    /// The top level: the drive roots, then whichever groups have members.
    pub fn roots(&self) -> &[Node] {
        &self.roots
    }

    /// The node a **position path** names — the index of a root, then of a
    /// child at each level below it.
    ///
    /// It is how the dialog turns the widget's selection back into an Entry:
    /// a native tree item carries no identity a caller may keep, but its place
    /// among its siblings is one the widget can always be asked for, and this
    /// tree never changes under it. An empty path names nothing, because there
    /// is no root node for it to name.
    pub fn at(&self, path: &[usize]) -> Option<&Node> {
        let (first, rest) = path.split_first()?;
        let mut node = self.roots.get(*first)?;
        for step in rest {
            node = node.children().get(*step)?;
        }
        Some(node)
    }
}

/// A node while it is still being built: the label as first spelled, the fold
/// that decides what merges into it, the Entry it stands for if it is a leaf,
/// and what has been merged beneath it.
///
/// A struct rather than [`Node`]'s enum because building is all mutation —
/// and the invariant [`Node`] states is kept by construction here: only a
/// branch is ever descended into, and only a leaf is ever given an entry.
struct Child {
    chain: String,
    key: String,
    entry: Option<Leaf>,
    children: Vec<Child>,
}

impl Child {
    fn leaf(chain: String, entry: Leaf) -> Child {
        Child {
            key: fold(&chain),
            chain,
            entry: Some(entry),
            children: Vec::new(),
        }
    }

    fn branch(chain: &str) -> Child {
        Child {
            chain: chain.to_string(),
            key: fold(chain),
            entry: None,
            children: Vec::new(),
        }
    }
}

/// Files `leaf` under the branch chain `above` names, creating what is missing
/// and merging into what is not.
///
/// **Only the segments above the Entry's last are merged**: the leaf itself is
/// always pushed, which is what makes duplicates two siblings and keeps an
/// Entry that is also a prefix reachable in its own right.
fn insert(children: &mut Vec<Child>, above: &[String], leaf: Child) {
    let Some((head, rest)) = above.split_first() else {
        children.push(leaf);
        return;
    };
    let key = fold(head);
    let index = match children
        .iter()
        .position(|child| child.entry.is_none() && child.key == key)
    {
        Some(index) => index,
        None => {
            // The first spelling wins the label; every later one merges into
            // it, because they name one directory.
            children.push(Child::branch(head));
            children.len() - 1
        }
    };
    insert(&mut children[index].children, rest, leaf);
}

/// Compresses each single-child chain, sorts every level, and hands back the
/// [`Node`]s the dialog builds from.
fn finish(children: Vec<Child>) -> Vec<Node> {
    let mut children: Vec<Child> = children.into_iter().map(compress).collect();
    // Stable, so the Working Copy's order breaks the ties the alphabet cannot
    // — which is what makes two duplicate leaves land in the order the list
    // shows their rows.
    children.sort_by(|left, right| left.key.cmp(&right.key));
    children.into_iter().map(node).collect()
}

/// Joins a branch with its only child, bottom-up (v0.2.0 §6).
///
/// The children are compressed first, so the one child a branch is left with
/// is already whole and one join finishes it. A branch that stands for an
/// Entry does not exist — leaves never have children — so there is nothing
/// here a compression could swallow.
fn compress(mut child: Child) -> Child {
    child.children = child.children.into_iter().map(compress).collect();
    if child.entry.is_some() || child.children.len() != 1 {
        return child;
    }
    let only = child.children.remove(0);
    let chain = format!("{}\\{}", child.chain, only.chain);
    Child {
        key: fold(&chain),
        chain,
        entry: only.entry,
        children: only.children,
    }
}

fn node(child: Child) -> Node {
    match child.entry {
        Some(entry) => Node::Leaf {
            chain: child.chain,
            entry,
        },
        None => Node::Branch {
            chain: child.chain,
            children: finish(child.children),
        },
    }
}

/// One path's segments, the root first: `C:`, or `\\server\share` for a UNC
/// path, and then each component between separators with the empties dropped.
///
/// The root is one segment because it is one place — `\\server` and `share`
/// are not two levels of anything a user browses, and a drive letter without
/// its separator is what lets the chains join with a plain `\`. Either
/// separator splits, as either qualifies a path for Win32.
fn segments(path: &str) -> Vec<String> {
    let path = path.replace('/', "\\");
    let mut parts = path.split('\\').filter(|part| !part.is_empty());
    let root = match path.strip_prefix(r"\\") {
        Some(_) => {
            let server = parts.next().unwrap_or_default();
            match parts.next() {
                Some(share) => format!(r"\\{server}\{share}"),
                None => format!(r"\\{server}"),
            }
        }
        None => parts.next().unwrap_or_default().to_string(),
    };
    std::iter::once(root)
        .chain(parts.map(str::to_string))
        .collect()
}

/// The reading that decides both what merges and how siblings sort:
/// Normalisation's own over one segment, which is the domain's answer to "are
/// these the same path?" — case folded, slash direction reconciled, a trailing
/// separator gone.
///
/// One fold for both questions on purpose: siblings that would merge are
/// siblings that sort together, so the alphabet can never separate two
/// spellings of one directory.
fn fold(segment: &str) -> String {
    Normalised::of_expanded(segment).as_str().to_string()
}
