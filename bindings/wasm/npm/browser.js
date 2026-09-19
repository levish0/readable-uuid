import init from './wasm/readable_uuid_wasm.js';

await init();

export * from './api.js';
