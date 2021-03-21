# NES Tetris Hard Drop Patcher

This tool reads an ines ROM file from standard input, assumes its NES Tetris,
and writes an ines ROM file to its standard output which is NES Tetris with
the addition of hard drop by pressing "up" on the controller.

```bash
$ cargo run < 'Tetris (U) [!].nes' > tetris-hard-drop.nes
$ fceux tetris-hard-drop.nes     # fceux is a NES emulator

```
![demo](/tetris-hard-drop-patcher/demo.gif)
