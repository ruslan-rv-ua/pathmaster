# Command surface: does v0.2.0 grow a toolbar?

Type: grilling
Status: open
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
