# pxlha

Match your [Home Assistant](https://www.home-assistant.io/) controlled RGB lights to the output of your screen.

## Requirements

Currently the project has a very strict set of requirements:

 - This only works on Linux systems with Wayland compositors.
 - The compositor must support [`zwlr_screencopy_v1`](https://wayland.app/protocols/wlr-screencopy-unstable-v1)
 - This only works for [Light](https://www.home-assistant.io/integrations/light/) entites on Home Assistant, that support the `hs_color` attribute.

## Building & Running

The usual Rust build steps

```sh
cargo build
```

Then edit the `.env.sample` file, and save as `.env`. You can then start the project with

```sh
cargo run
```

## Acknowledgements

This code is heavily based upon the [wayshot](https://github.com/waycrate/wayshot) project.