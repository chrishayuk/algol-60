// Builds the files the playground needs:  node web/build.mjs
//   web/algol60.wasm   the interpreter, compiled for WASI
//   web/examples.json  the example programs, with their input, and the
//                      outer-planets program with what the X1 printed in 1964
import { execSync } from "node:child_process";
import { copyFileSync, existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";

const root = new URL("..", import.meta.url).pathname;
execSync("cargo build --release --target wasm32-wasip1", { cwd: root, stdio: "inherit" });
copyFileSync(`${root}target/wasm32-wasip1/release/algol60.wasm`, `${root}web/algol60.wasm`);

// Shown first, in this order; the rest follow alphabetically.
const first = ["hello", "jensen", "man-or-boy-1964", "evaluation-count", "assignment-order", "x1-probes", "jaz164"];

const read = (path) => (existsSync(path) ? readFileSync(path, "utf8") : "");
const examples = readdirSync(`${root}examples`).filter((f) => f.endsWith(".alg")).map((f) => {
  const name = f.slice(0, -4);
  return { name, program: read(`${root}examples/${f}`), input: read(`${root}examples/${name}.in`) };
});
examples.push({
  name: "jaz164",
  program: read(`${root}historical/jaz164.alg`),
  input: read(`${root}historical/jaz164.in`),
  printed1964: read(`${root}historical/jaz164.1964.out`),
});
examples.sort((a, b) => {
  const [i, j] = [first.indexOf(a.name), first.indexOf(b.name)];
  if (i !== -1 || j !== -1) return (i === -1 ? 99 : i) - (j === -1 ? 99 : j);
  return a.name.localeCompare(b.name);
});
writeFileSync(`${root}web/examples.json`, JSON.stringify(examples));
console.log(`web/algol60.wasm and web/examples.json (${examples.length} programs) written`);
