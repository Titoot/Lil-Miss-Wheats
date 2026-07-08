# Lil' Miss Wheats

A GUI editor for MSBT string table files, made for translating **Rhythm Heaven Groove**.

## Features

- Open, edit, and save `.msbt` files
- Tag-aware editor with support for:
  - `[TTS("...")]` — TTS voice text + display text segments
  - `[SEP]` — textbox separator
  - `[N]` — null byte / line break
  - `[A]` `[Y]` `[+]` `[MINUS]` `[LEFT]` `[DOWN]` `[RIGHT]` — button icon tags
- Arabic text shaping and reversal for in-game display
- Real-time hex preview of entry bytes
- Original text reference panel (uneditable, for comparison)
- Dark mode UI

## Building

Requires [GTK4](https://gtk.org) development libraries.

### Windows (MSYS2)

```bash
pacman -S mingw-w64-x86_64-gtk4
$env:Path = "C:\msys64\mingw64\bin;" + $env:Path
$env:GSETTINGS_SCHEMA_DIR = "C:\msys64\mingw64\share\glib-2.0\schemas"
cargo build
```

### Linux

```bash
sudo apt install libgtk-4-dev pkg-config
cargo build
```
