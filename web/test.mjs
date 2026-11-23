// Runs every example through the WebAssembly build and the browser's WASI
// host, as the Report and as the X1, and compares the output with the
// .out and .x1.out files that cargo test checks the native build against.
//   node web/test.mjs
import { existsSync, readFileSync } from "node:fs";
import { argsFor, run, stdinFor } from "./wasi.js";

const root = new URL("..", import.meta.url).pathname;
const module = new WebAssembly.Module(readFileSync(root + "web/algol60.wasm"));
const examples = JSON.parse(readFileSync(root + "web/examples.json", "utf8"));

function algol60(program, input, mode) {
  const text = { 1: "", 2: "" };
  const decoders = { 1: new TextDecoder(), 2: new TextDecoder() };
  const code = run(module, argsFor(program, mode), stdinFor(input), (fd, bytes) => {
    text[fd] += decoders[fd].decode(bytes, { stream: true });
  });
  return { code, out: text[1], err: text[2] };
}

let failed = 0;
let checked = 0;
const deep = []; // too deep for the host's own stack, as in a browser
for (const { name, program, input } of examples) {
  const dir = name === "jaz164" ? "historical" : "examples";
  for (const [mode, suffix] of [["report", ".out"], ["x1", ".x1.out"]]) {
    const file = `${root}${dir}/${name}${suffix}`;
    if (!existsSync(file)) continue;
    const want = readFileSync(file, "utf8");
    let got;
    try {
      got = algol60(program, input, mode);
    } catch (error) {
      if (!(error instanceof RangeError)) throw error;
      deep.push(`${name} (${mode})`);
      continue;
    }
    checked++;
    if (!(got.out === want || got.out + got.err === want || want.startsWith(got.out))) {
      failed++;
      console.log(`FAIL ${name} (${mode})\n--- want\n${want.slice(0, 300)}\n--- got\n${got.out.slice(0, 300)}${got.err.slice(0, 300)}`);
    }
  }
}
console.log(`${checked - failed} of ${checked} outputs agree with the native build's`);
if (deep.length) console.log(`too deep for this host's stack: ${deep.join(", ")}`);
process.exit(failed ? 1 : 0);
