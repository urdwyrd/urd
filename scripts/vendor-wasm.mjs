/**
 * Copies WASM artifacts from packages/compiler/pkg/ to the Astro site.
 *
 * - .wasm binary  → sites/urd.dev/public/wasm/  (static asset)
 * - JS/TS glue    → sites/urd.dev/src/lib/wasm/  (imported by bridge)
 */

import { cpSync, mkdirSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const pkg = resolve(root, 'packages/compiler/pkg');
const publicWasm = resolve(root, 'sites/urd.dev/public/wasm');
const libWasm = resolve(root, 'sites/urd.dev/src/lib/wasm');

if (!existsSync(pkg)) {
  console.error('Error: packages/compiler/pkg/ not found. Run pnpm compiler:wasm:build first.');
  process.exit(1);
}

mkdirSync(publicWasm, { recursive: true });
mkdirSync(libWasm, { recursive: true });

// Binary → public (served as static asset)
cpSync(resolve(pkg, 'urd_compiler_bg.wasm'), resolve(publicWasm, 'urd_compiler_bg.wasm'));

// JS glue + TypeScript declarations → src/lib (imported by bridge module)
// Patch the JS glue to suppress Vite's build-time URL warning (the fallback
// path is never hit because the bridge passes an explicit fetch URL).
const jsGlue = readFileSync(resolve(pkg, 'urd_compiler.js'), 'utf-8');
const patched = jsGlue.replace(
  "new URL('urd_compiler_bg.wasm', import.meta.url)",
  "new URL(/* @vite-ignore */ 'urd_compiler_bg.wasm', import.meta.url)"
);
writeFileSync(resolve(libWasm, 'urd_compiler.js'), patched);
cpSync(resolve(pkg, 'urd_compiler.d.ts'), resolve(libWasm, 'urd_compiler.d.ts'));
cpSync(resolve(pkg, 'urd_compiler_bg.wasm.d.ts'), resolve(libWasm, 'urd_compiler_bg.wasm.d.ts'));

console.log('Vendored WASM artifacts:');
console.log('  → sites/urd.dev/public/wasm/urd_compiler_bg.wasm');
console.log('  → sites/urd.dev/src/lib/wasm/urd_compiler.js');
console.log('  → sites/urd.dev/src/lib/wasm/urd_compiler.d.ts');
console.log('  → sites/urd.dev/src/lib/wasm/urd_compiler_bg.wasm.d.ts');
