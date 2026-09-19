import { readFile } from 'node:fs/promises';
import init from './wasm/readable_uuid_wasm.js';

const moduleBytes = await readFile(new URL('./wasm/readable_uuid_wasm_bg.wasm', import.meta.url));

await init({ module_or_path: moduleBytes });

export * from './api.js';
