import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import { performance } from 'node:perf_hooks';
import { entropyToMnemonic, mnemonicToEntropy } from '@scure/bip39';
import { wordlist } from '@scure/bip39/wordlists/english.js';
import niceware from 'niceware';
import { decode, decodeBatch, encode, encodeBatch } from '../pkg-npm/index.js';

function uuidFromBytes(bytes: Uint8Array): string {
  const hex = Buffer.from(bytes).toString('hex');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

function uuidBytes(uuid: string): Buffer {
  return Buffer.from(uuid.replaceAll('-', ''), 'hex');
}

const uuids = Array.from({ length: 4096 }, (_, index) => {
  const bytes = createHash('sha256')
    .update(`readable-uuid-bench:${index}`)
    .digest()
    .subarray(0, 16);
  if (index % 2) {
    bytes.writeUIntBE(1760000000000, 0, 6);
    bytes[6] = (bytes[6] & 15) | 0x70;
  } else {
    bytes[6] = (bytes[6] & 15) | 0x40;
  }
  bytes[8] = (bytes[8] & 63) | 0x80;
  return uuidFromBytes(bytes);
});

interface Codec {
  name: string;
  encode: (uuid: string) => string;
  decode: (phrase: string) => string;
  checksum: boolean;
}

const codecs: Codec[] = [
  {
    name: 'readable-uuid',
    encode: (uuid) => encode(uuid, { separator: ' ' }),
    decode: (phrase) => decode(phrase, { separator: ' ' }),
    checksum: false,
  },
  {
    name: 'niceware',
    encode: (uuid) => niceware.bytesToPassphrase(uuidBytes(uuid)).join(' '),
    decode: (phrase) => uuidFromBytes(niceware.passphraseToBytes(phrase.split(' '))),
    checksum: false,
  },
  {
    name: '@scure/bip39',
    encode: (uuid) => entropyToMnemonic(uuidBytes(uuid), wordlist),
    decode: (phrase) => uuidFromBytes(mnemonicToEntropy(phrase, wordlist)),
    checksum: true,
  },
];

// Prepare decoder inputs and verify every round trip outside the timed region.
const phrases = codecs.map((codec) => {
  const encoded = uuids.map(codec.encode);
  assert.deepEqual(encoded.map(codec.decode), uuids, `${codec.name} round trips`);
  assert.equal(new Set(encoded).size, uuids.length);
  return encoded;
});

const operationsPerRound = 16384;
const rounds = 15;
let checksum = 0;

function measure(operation: (index: number) => string): number {
  const start = performance.now();
  for (let index = 0; index < operationsPerRound; index++) {
    checksum = (checksum + operation(index % uuids.length).length) | 0;
  }
  return ((performance.now() - start) * 1e6) / operationsPerRound;
}

const cases = codecs.flatMap((codec, codecIndex) => [
  {
    name: codec.name,
    direction: 'encode',
    operation: (index: number) => codec.encode(uuids[index]),
    samples: [] as number[],
  },
  {
    name: codec.name,
    direction: 'decode',
    operation: (index: number) => codec.decode(phrases[codecIndex][index]),
    samples: [] as number[],
  },
]);

for (const benchmark of cases) {
  for (let warmup = 0; warmup < 3; warmup++) {
    measure(benchmark.operation);
  }
}

for (let round = 0; round < rounds; round++) {
  for (let offset = 0; offset < cases.length; offset++) {
    const benchmark = cases[(round + offset) % cases.length];
    benchmark.samples.push(measure(benchmark.operation));
  }
}

function statistics(samples: number[]) {
  const sorted = [...samples].sort((left, right) => left - right);
  return {
    medianNs: Math.round(sorted[Math.floor(sorted.length / 2)]),
    p10Ns: Math.round(sorted[Math.floor(sorted.length * 0.1)]),
    p90Ns: Math.round(sorted[Math.floor(sorted.length * 0.9)]),
  };
}

const results = cases.map(({ name, direction, samples }) => ({
  name,
  direction,
  ...statistics(samples),
}));

const batch = uuids.slice(0, 1000);
const batchPhrases = encodeBatch(batch);
assert.deepEqual(decodeBatch(batchPhrases), batch);
const batchResults = [
  { direction: 'encode', operation: () => encodeBatch(batch) },
  { direction: 'decode', operation: () => decodeBatch(batchPhrases) },
].map(({ direction, operation }) => {
  for (let warmup = 0; warmup < 100; warmup++) {
    operation();
  }
  const samples = Array.from({ length: rounds }, () => {
    const start = performance.now();
    for (let iteration = 0; iteration < 100; iteration++) {
      checksum = (checksum + operation()[0].length) | 0;
    }
    return ((performance.now() - start) * 1e6) / 100000;
  });
  return { direction, ...statistics(samples) };
});

function packageVersion(relativePath: string): string {
  return (
    JSON.parse(readFileSync(new URL(relativePath, import.meta.url), 'utf8')) as {
      version: string;
    }
  ).version;
}

const report = {
  generatedAt: new Date().toISOString(),
  node: process.version,
  platform: process.platform,
  arch: process.arch,
  cpu: os.cpus()[0].model,
  versions: {
    'readable-uuid': packageVersion('../pkg-npm/package.json'),
    niceware: packageVersion('../node_modules/niceware/package.json'),
    '@scure/bip39': packageVersion('../node_modules/@scure/bip39/package.json'),
  },
  corpus: {
    count: uuids.length,
    description: 'Deterministic SHA256 corpus: half UUIDv4, half same-millisecond UUIDv7',
  },
  scope: 'Canonical UUID string <-> space-separated phrase; parsing and joining included',
  rounds,
  operationsPerRound,
  outputs: codecs.map((codec, index) => ({
    name: codec.name,
    words: phrases[index][0].split(' ').length,
    meanCharacters: Number(
      (phrases[index].reduce((total, phrase) => total + phrase.length, 0) / uuids.length).toFixed(
        2,
      ),
    ),
    checksum: codec.checksum,
  })),
  results,
  batch: {
    name: 'readable-uuid',
    size: batch.length,
    unit: 'ns per UUID',
    results: batchResults,
  },
  checksum,
};
writeFileSync(new URL('results.json', import.meta.url), JSON.stringify(report, null, 2) + '\n');
console.table(report.outputs);
console.table(results);
console.table(batchResults);
