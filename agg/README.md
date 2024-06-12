# agg - asciinema gif generator

Heavily modified fork of the [original](https://github.com/asciinema/agg).

agg is a command-line tool for generating animated GIF files from
[asciicast v2 files](https://github.com/asciinema/asciinema/blob/master/doc/asciicast-v2.md)
produced by [asciinema terminal recorder](https://github.com/asciinema/asciinema).

It uses Kornel Lesiński's excellent [gifski](https://github.com/ImageOptim/gifski)
library to produce optimized, high quality GIF output with accurate frame timing.

# Building

Building from source requires [Rust](https://www.rust-lang.org/) compiler
(1.56.0 or later) and [Cargo package manager](https://doc.rust-lang.org/cargo/).
You can install both with [rustup](https://rustup.rs/).

```bash
just install-agg
```

# Usage

Basic usage:

```bash
agg demo.cast demo.gif
```

The above command renders a GIF file with default theme (dracula), font size 14px.

Additional options are available for customization. For example, the following
command selects Monokai theme, larger font size, 2x playback speed:

```bash
agg --theme monokai --font-size 20 --speed 2 demo.cast demo.gif
```

Run `agg -h` to see all available options. Current options are:

```text
--renderer <RENDERER>
    Select frame rendering backend [default: fontdue] [possible values: fontdue, resvg]

--font <FONT>
    Specify font family [default: "Monaspace Neon" "Symbols Nerd Font" "JetBrainsMono Nerd Font"]

--font-dir <FONT_DIR>
    Use additional font directory

--font-size <FONT_SIZE>
    Specify font size (in pixels) [default: 14]

--line-height <LINE_HEIGHT>
    Specify line height [default: 1.4]

--theme <THEME>
    Select color theme [default: dracula] [possible values: asciinema, dracula, github-dark, github-light, monokai, nord, solarized-dark, solarized-light]

--speed <SPEED>
    Adjust playback speed [default: 1]

--fps-cap <FPS_CAP>
    Set FPS cap [default: 30]

--last-frame-duration <LAST_FRAME_DURATION>
    Set last frame duration [default: 1]

-v, --verbose
    Enable verbose logging

-h, --help
    Print help information

-V, --version
    Print version information
```

# Fonts

If you want to use another font family then pass multiple values like this:

```bash
agg --font "Source Code Pro" --font "Fira Code" demo.cast demo.gif
```

As long as the fonts you want to use are installed in one of standard system
locations (e.g. /usr/share/fonts or ~/.local/share/fonts on Linux) agg will find
them. You can also use `--font-dir=/path/to/fonts` option to include extra
fonts. `--font-dir` can be specified multiple times.

To verify agg picks up your font run it with `-v` (verbose) flag:

```bash
agg -v --font "Source Code Pro" --font "Fira Code" demo.cast demo.gif
```

It should print something similar to:

```text
[INFO agg] selected font families: ["Source Code Pro", "Fira Code", "DejaVu Sans", "Noto Emoji"]
```

This list may also include implicit addition of DejaVu Sans fallback (mentioned
earlier), as well as Noto Emoji (see section below).

# Color themes

There are several built-in color themes you can use with `--theme` option:

- asciinema
- dracula (default)
- monokai
- solarized-dark
- solarized-light
