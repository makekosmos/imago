# imago-nsh

`imago-nsh` provides the GPUI host, Engine client, shared NSH theme and reusable TSX components. The npm package and Cargo crate live at the same Imago revision.

## Connect an app

1. Pin the same Imago revision in Cargo and the app's UI package:

   ```toml
   imago-nsh = { git = "https://github.com/makekosmos/imago.git", rev = "<revision>" }
   ```

   ```json
   { "dependencies": { "@makekosmos/imago-nsh": "git+https://github.com/makekosmos/imago.git#<revision>" } }
   ```

2. Install the UI dependency with `npm install` in the directory containing `nsh.json`.
3. Import the shared shell and controls in TSX:

   ```tsx
   import { Sidebar, Titlebar } from "@makekosmos/imago-nsh";
   ```

4. Build the TSX entry with `nsh build ui` and mount the bundle in the GPUI window:

   ```rust
   mount_app_view(window, cx, NshApp::Js(Bundle::source(bundle)), |_| {})
   ```

`examples/minimal` shows a window shell, sidebar and one page. The shared package exports theme tokens, SVG icons, titlebar, sidebar navigation, common controls and appearance settings. An application supplies its navigation entries, theme colors and page content.
