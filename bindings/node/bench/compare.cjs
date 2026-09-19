const { performance } = require('node:perf_hooks');
const { createHash } = require('node:crypto');
const { readFileSync, writeFileSync } = require('node:fs');
const { resolve } = require('node:path');
const os = require('node:os');
const { readableUuid, readableUuidBatch } = require('../pkg-npm/index.js');
const HumanHasher = require('humanhash');
const WordHash = require('wordhash');

const ids = Array.from({ length: 4096 }, (_, i) => {
  const bytes = createHash('sha256').update(`readable-uuid-bench:${i}`).digest().subarray(0, 16);
  if (i % 2) {
    bytes.writeUIntBE(1760000000000, 0, 6);
    bytes[6] = (bytes[6] & 15) | 0x70;
  } else {
    bytes[6] = (bytes[6] & 15) | 0x40;
  }
  bytes[8] = (bytes[8] & 63) | 0x80;
  const hex = bytes.toString('hex');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
});

const hasher = new HumanHasher();
const wordhash = WordHash({ length: 4, separator: '-' });

const cases = [
  { name: 'readable-uuid', group: 'UUID to four words', fn: (id) => readableUuid(id) },
  {
    name: 'humanhash',
    group: 'UUID to four words',
    fn: (id) => hasher.humanize(id.replaceAll('-', ''), 4, '-'),
  },
  { name: 'wordhash', group: 'UUID to four words', fn: (id) => wordhash.hash(id) },
];

let checksum = 0;

function timed(fn, operations = 16384) {
  const start = performance.now();
  for (let i = 0; i < operations; i++) {
    checksum = (checksum + fn(ids[i % ids.length]).length) | 0;
  }
  return ((performance.now() - start) * 1e6) / operations;
}

for (const item of cases) {
  for (let i = 0; i < 3; i++) {
    timed(item.fn);
  }
}

const samples = new Map(cases.map((item) => [item.name, []]));
// Rotate order each round so one implementation does not always run first.
for (let round = 0; round < 15; round++) {
  for (let i = 0; i < cases.length; i++) {
    const item = cases[(round + i) % cases.length];
    samples.get(item.name).push(timed(item.fn));
  }
}

const stats = (values) => {
  const sorted = [...values].sort((a, b) => a - b);
  return {
    medianNs: Math.round(sorted[Math.floor(sorted.length / 2)]),
    p10Ns: Math.round(sorted[Math.floor(sorted.length * 0.1)]),
    p90Ns: Math.round(sorted[Math.floor(sorted.length * 0.9)]),
  };
};
const results = cases.map((item) => {
  const labels = ids.map(item.fn);
  return {
    name: item.name,
    group: item.group,
    ...stats(samples.get(item.name)),
    meanCharacters: Number((labels.reduce((n, s) => n + s.length, 0) / labels.length).toFixed(2)),
    emptyWordLabels: labels.filter((s) => s.split('-').some((word) => !word)).length,
    uniqueLabels: new Set(labels).size,
  };
});

const batch = ids.slice(0, 1000);
for (let i = 0; i < 100; i++) {
  readableUuidBatch(batch);
}

const batchSamples = [];
for (let round = 0; round < 15; round++) {
  const start = performance.now();
  for (let i = 0; i < 100; i++) {
    const labels = readableUuidBatch(batch);
    checksum = (checksum + labels[0].length) | 0;
  }
  batchSamples.push(((performance.now() - start) * 1e6) / 100000);
}
results.push({
  name: 'readable-uuid (batch 1000)',
  group: 'Batch; ns per UUID',
  ...stats(batchSamples),
});

const versions = Object.fromEntries(
  ['humanhash', 'wordhash'].map((name) => [
    name,
    JSON.parse(readFileSync(resolve(__dirname, '../node_modules', name, 'package.json'))).version,
  ]),
);
versions['readable-uuid'] = JSON.parse(
  readFileSync(resolve(__dirname, '../pkg-npm/package.json')),
).version;
const report = {
  generatedAt: new Date().toISOString(),
  node: process.version,
  platform: process.platform,
  arch: process.arch,
  cpu: os.cpus()[0].model,
  versions,
  corpus: {
    count: ids.length,
    description: 'Deterministic SHA256 corpus: half UUIDv4, half same-millisecond UUIDv7',
  },
  rounds: 15,
  operationsPerRound: 16384,
  results,
  checksum,
};
writeFileSync(resolve(__dirname, 'results.json'), JSON.stringify(report, null, 2) + '\n');
console.table(results);
