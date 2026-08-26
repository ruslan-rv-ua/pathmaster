# %VAR% expansion display toggle

Type: grilling
Status: open
Blocked by: —

## Question

FR-var-expansion-toggle: a command flips the list between raw entries (`%JAVA_HOME%\bin`) and
expanded ones (`C:\jdk21\bin`). v0.1.0 already expands at comparison time (Normalisation), never for
display. Decide the display-mode contract:

- Is the mode per-Scope or app-wide? Does it persist in `settings.json` or reset per Run?
- The PRD says the mode change is not an edit (no dirty state) — confirm, and decide whether it's a
  Checkpoint no-op too.
- What exactly is displayed in expanded mode for an entry whose `%VAR%` is undefined? The PRD invents
  a "Warning: Unknown variable" marker — but v0.1.0 has no severity classes, and an undefined var
  already flags `Missing` naturally (spec §7). Does the toggle need any new Issue type at all, or
  does the existing Status column already say everything?
- Editing while expanded: the Edit dialog edits the **raw** text (the stored truth). Confirm, and
  decide what the list shows mid-edit.
- What is announced on toggle (new member of the closed Announcement set — exact wording, both
  languages), and how does the current mode remain discoverable to NVDA afterwards?
- Interaction with Search: recorded here but decided in the Search ticket — expansion mode changes
  the visible text, so "search over which text" must not be decided twice.
