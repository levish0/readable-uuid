import {
  decode as decodeWasm,
  decodeBatch as decodeBatchWasm,
  encode as encodeWasm,
  encodeBatch as encodeBatchWasm,
} from './wasm/readable_uuid_wasm.js';

function separator(options) {
  if (options === undefined) {
    return undefined;
  }
  if (options === null || typeof options !== 'object') {
    throw new TypeError('options must be an object');
  }
  return options.separator;
}

export function encode(uuid, options) {
  return encodeWasm(uuid, separator(options));
}

export function decode(phrase, options) {
  return decodeWasm(phrase, separator(options));
}

export function encodeBatch(uuids, options) {
  return encodeBatchWasm(uuids, separator(options));
}

export function decodeBatch(phrases, options) {
  return decodeBatchWasm(phrases, separator(options));
}
