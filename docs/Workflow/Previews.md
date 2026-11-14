# Previews

See a component or a screen in all its states, with fixed data, without running the simulator. Previews are written **before** the component is finished: they are the demo the component has to match (demo-driven development, like Storybook or Xcode Previews).

```sh
just preview components/primary-button                          # gallery of ui/components/primary-button.preview.slint
just preview components/primary-button PrimaryButtonPlayground  # one story of that file
just preview screens/history HistoryMainEmpty                   # a Pencil frame, to compare side by side
just preview app-window                                         # the real app window, routing included
```
Needs `slint-viewer` (`cargo install slint-viewer`); it reloads on every save. In VS Code, the Slint extension's **Show Preview** above any component does the same, and its **Data** tab edits the story's `in-out` properties live.

## Pure components
A preview is only worth something if the component can be fed fake data. So components in `components/`, `shell/` and `screens/`:
- receive every piece of state through `in` / `in-out` properties and report every action through `callback`s;
- never read an app-state global; only `Theme` (tokens) and asset globals (`RoleIcons`) are allowed;
- take structured data as structs declared in `ui/models.slint` (created with the first struct), never as Rust-shaped globals.

Only `app-window.slint` binds the app state (set by Rust) to the screens. `shell/app-shell.slint` (top bar + current screen) is shared by the app window and the screen previews, so a preview composes a screen exactly as the app does.

## Layout
Each preview sits **next to** the file it previews, with the same name and a `.preview.slint` suffix:
```
ui/
  app-window.slint
  app-window.preview.slint      the whole app, fed with fixtures
  components/
    primary-button.slint
    primary-button.preview.slint
  screens/
    history.slint
    history.preview.slint
  preview-kit/                  helpers for previews only
    preview-case.slint          PreviewCase: a caption above one variant
    preview-playground.slint    PreviewPlayground: frame of a playground, logs callbacks
    fixtures.slint              Fixtures: shared sample data (status labels, version, …)
```
The app never imports a `.preview.slint` file or `preview-kit/`; `build.rs` only compiles `app-window.slint` and what it imports, so previews never reach the binary.

## A preview file
It exports several components, called **stories**, in this order:
1. **Pencil stories**: one per Pencil frame the item implements, named after the frame in PascalCase (`History / Main / Empty` → `HistoryMainEmpty`). It inherits `Rectangle` at the frame's exact size and shows what the frame shows (a screen inside `AppShell` when the frame has the top bar), with the frame's data. This is what you compare 1:1 with Pencil.
2. **Playground** (`<Component>Playground`), when the component has its own `in` properties or callbacks. It inherits `Window`, exposes the component's inputs as `in-out` properties with realistic defaults (the knobs of the Data tab), and calls `playground.log("<callback>")` from every callback so clicks show up as "Last action".
3. **`Preview`**, the gallery, **always last**: slint-viewer opens the last exported component, so `just preview <path>` shows it. It inherits `Window` and stacks every variant in a `PreviewCase`: the Pencil stories, then the default, short and long text, empty values, a narrow width, and every state (connected or not, active or not, error).

Preview files are the one exception to "main item first" (see [[Code organisation]]): the gallery uses the stories, so it has to come after them. `app-window.preview.slint` is the exception to the exception: its `Preview` simply inherits `AppWindow` with fixture values.

## Data
- Use `Fixtures.*` for values shared by several previews, and keep them identical to what Rust sends (e.g. status labels from `status_label` in `src/watcher.rs`). One-off values stay inline.
- A preview that needs the simulator isn't a preview: use `just dev` and a scenario ([[Simulator]]).

## Demo-driven loop
1. Read the feature note: list the states of the screen or component.
2. Check each state has a Pencil frame ([[Design with Pencil]]).
3. Write the preview first: one Pencil story per frame, the playground, the gallery with the edge cases. It can start with placeholder content.
4. Implement the component until each Pencil story matches its frame (`just preview <path> <Story>` next to Pencil), then wire it in `app-window.slint`.

## Checks
`cargo test -p cooldown-app --test previews` (part of `just test` and CI) fails when:
- a file of `components/`, `shell/`, `screens/` or `app-window.slint` has no `.preview.slint` next to it, or a preview has no component file next to it;
- a preview doesn't compile;
- a preview's last exported component isn't `Preview`;
- an app file mentions a `.preview.slint` file or `preview-kit/`.

A component change that breaks a preview fixes it in the same change.
