# Command surface: does v0.2.0 grow a toolbar?

Type: grilling
Status: resolved (2026-08-26)
Blocked by: —

## Question

The PRD puts several v0.2.0 features "in the toolbar" (Fix Issues, Tree View, the Expand %VAR%
toggle). v0.1.0 shipped **no toolbar and no in-app iconography** (spec §12), and its standing rule is
that every command has a menu home because a menu item's label is the only place wxdragon can carry a
shortcut. Decide the command-surface model for v0.2.0 before any feature ticket assumes one:

- Does v0.2.0 introduce a toolbar at all, or do all new commands live in menus + accelerators, as
  every v0.1.0 command does?
- If no toolbar: which menus are the natural homes (PRD sketched a `View` menu that v0.1.0 never
  built — does v0.2.0 create it?), and does the "reduced structure" principle from v0.1.0's menu
  design survive the growth?
- If a toolbar: what does that do to the no-iconography rule, tab order, and NVDA navigation — and
  what does it buy a keyboard-first user?

This ticket decides the *model*; the exact menu/accelerator table stays in the fog until the feature
contracts land (see the map's Not yet specified).

## Resolution (2026-08-26)

**No toolbar. The View menu returns. Commands sort by what they change.**

1. **v0.2.0 grows no toolbar** and no in-app iconography; spec §12 / D8 stand unchanged. The PRD's
   three toolbar placements (FR-var-expansion-toggle, FR-tree-browser, FR-fix-issues) are recorded
   as deviations: each feature gets a menu home instead, losing nothing.
2. **The menu bar becomes File / Edit / View / Tools / Help.** Reinstating View is the
   reduced-structure principle honestly applied, not reversed: v0.1.0 cut the menu because its
   entire PRD contents (Filter, Tree View, Search) were 🟡 features cut to v0.2.0 — v0.2.0 ships
   exactly those features, so the menu returns with them.
3. **The model**: commands that change *what the list shows* (Search, Tree View, Expand %VAR%, and
   the filter if ticket 07 chooses a menu representation) live in **View**; commands that change
   the *Working Copy* live in **Edit**; everything else follows v0.1.0. Exact item order,
   accelerators and mnemonics stay in the fog for the feature tickets and the assembly ticket 15
   (which re-runs the mnemonic gate in both languages over all menu growth at once).
4. **Standing rules unchanged**: every shortcut has a menu home (a menu item's label is the only
   place wxdragon can carry one); no shortcut is ever typed into a translated string (ADR-0004).

**Evidence** (why either outcome of "toolbar?" was not a coin flip):

- Microsoft's keyboard-UI accessibility guidance states toolbars are "generally not accessible to
  keyboard users" and requires every toolbar function to exist as a menu/shortcut equivalent
  ([Guidelines for Keyboard UI Design](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/dnacc/guidelines-for-keyboard-user-interface-design));
  the Windows UX guide says to prefer retaining the menu bar for exactly this reason
  ([Toolbars](https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-toolbars)).
- The NVDA User Guide treats toolbars as the canonical case of UI reached only via object
  navigation, not Tab; the ToolbarsExplorer add-on exists solely to ease that pain. NVDA's own GUI
  (wxPython) has no toolbar.
- wxToolBar on MSW documents no keyboard interface at all — tools are mouse-only without custom
  handling — and toolbars are absent from ADR-0003's list of what NVDA reads free from native
  widgets, so a toolbar would have needed its own NVDA prototype ticket.
- Technically it was reachable (wxdragon `Frame::create_tool_bar`, tools deliver `EventType::MENU`
  in the same id namespace as menus, `ToolBarStyle::Text` to stay text-only) — rejected on product
  identity, not feasibility: it buys nothing for the keyboard-first user and adds unverified
  accessibility surface, new tab-order contract, and a Release Checklist step.

No new tickets; no fog graduates (the menu/accelerator/Announcement assembly sharpens only once
feature contracts land). No ADR: easily reversed, unsurprising given the product, and the evidence
left no genuine trade-off. Downstream tickets 05, 06, 07, 08, 09, 11, 12 consume this model as
"per 02's model".
