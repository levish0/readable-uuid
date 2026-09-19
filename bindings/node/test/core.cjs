const test = require('node:test');
const assert = require('node:assert/strict');
const { readableUuid, readableUuidBatch, wordSets } = require('../pkg-npm/index.js');
const fixtures = require('../../../tests/vectors.json');
test('shared Rust golden vectors', () => {
  for (const v of fixtures.vectors) {
    const options = { wordSet: v.set, words: v.words, separator: v.separator };
    assert.equal(readableUuid(v.uuid, options), v.label);
    assert.deepEqual(readableUuidBatch([v.uuid, v.uuid.toUpperCase()], options), [
      v.label,
      v.label,
    ]);
  }
});
test('errors, custom dictionaries and metadata', () => {
  for (const words of [-1, 1.5, NaN, Infinity, 2 ** 32 + 4]) {
    assert.throws(() => readableUuid('00000000-0000-0000-0000-000000000000', { words }));
  }
  for (const options of [
    { words: 0 },
    { words: 65 },
    { wordSet: 'missing' },
    { customWords: [] },
    { customWords: ['a', 'a'] },
    { separator: '' },
    { wordSet: 'english-v1', customWords: ['a', 'b'] },
  ]) {
    assert.throws(() => readableUuid('00000000-0000-0000-0000-000000000000', options));
  }
  assert.throws(() => readableUuid('not a UUID'));
  assert.throws(() => readableUuidBatch(['00000000-0000-0000-0000-000000000000', 'bad']));
  assert.deepEqual(readableUuidBatch([]), []);
  const options = { customWords: ['red', 'green', 'blue'], words: 6 };
  const label = readableUuid('00000000-0000-0000-0000-000000000000', options);
  assert.match(label, /^(red|green|blue)(-(red|green|blue)){5}$/);
  assert.equal(wordSets().length, 3);
});
