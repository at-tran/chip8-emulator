import * as wasm from "../pkg/index.js";
import "../sass/style.sass";

// This is to prevent ../pkg/index.js from being excluded
// when building for production
wasm;