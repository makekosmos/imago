# imago-nsh

`imago-nsh` gives a GPUI app the shared NSH host, Imago theme crate, and Mundus Engine client through one dependency. NSH screens remain TSX modules in the consuming app.

Add the dependency at the Imago revision you use:

```toml
imago-nsh = { git = "https://github.com/makekosmos/imago.git", rev = "<revision>" }
```

Mount the TSX bundle in the window builder and register the app's native bridge before loading it:

```rust
let view = imago_nsh::mount_app_view(
    window,
    cx,
    imago_nsh::NshApp::Js(imago_nsh::Bundle::source(include_str!("../ui/bundle.js").into())),
    |view| view.register_component("app.bridge", bridge_factory()),
);
```

The consuming crate can reach the existing Engine client and theme APIs as `imago_nsh::mundus_gpui_kit` and `imago_nsh::imago_gpui`. NSH's base components use `imago_nsh::nsh_base_registry()` and `imago_nsh::NshTheme`.

This crate currently shares the host and Rust APIs. TSX titlebars, navigation, controls, and appearance screens still belong to each application.
