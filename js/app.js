import * as wasm from "../pkg/index.js";
import "../sass/style.sass";

let chip8 = wasm.Chip8Emulator.new();
fetch("roms/15PUZZLE")
    .then(r => r.arrayBuffer())
    .then(arr => {
        chip8.load_rom(new Uint8Array(arr));

    });