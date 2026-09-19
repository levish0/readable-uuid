import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { decode, decodeBatch, encode, encodeBatch } from '../pkg-npm/index.js';

const nil = '00000000-0000-0000-0000-000000000000';
const maximum = 'ffffffff-ffff-ffff-ffff-ffffffffffff';
interface Vector {
  uuid: string;
  separator: string;
  label: string;
}
const fixtures = JSON.parse(
  readFileSync(new URL('../../../tests/vectors.json', import.meta.url), 'utf8'),
) as { vectors: Vector[] };

test('shared Rust golden vectors in both directions', () => {
  for (const vector of fixtures.vectors) {
    const options = { separator: vector.separator };
    assert.equal(encode(vector.uuid, options), vector.label);
    assert.equal(decode(vector.label, options), vector.uuid.toLowerCase());
    assert.deepEqual(encodeBatch([vector.uuid, vector.uuid.toUpperCase()], options), [
      vector.label,
      vector.label,
    ]);
    assert.deepEqual(decodeBatch([vector.label, vector.label], options), [
      vector.uuid.toLowerCase(),
      vector.uuid.toLowerCase(),
    ]);
  }
});

test('independent byte reference and deterministic built-in round trips', () => {
  const readWords = (filename: string) =>
    readFileSync(new URL(`../../../wordlists/${filename}`, import.meta.url), 'utf8')
      .trim()
      .split(/\r?\n/);
  const modifiers = readWords('modifiers.txt');
  const nouns = readWords('nouns.txt');
  assert.equal(modifiers.length, 256);
  assert.equal(nouns.length, 256);
  const uuids = [nil, maximum];
  for (let index = 0; index < 256; index++) {
    const hex = createHash('sha256').update(`codec-test:${index}`).digest('hex').slice(0, 32);
    uuids.push(
      `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`,
    );
  }
  for (const uuid of uuids) {
    const bytes = Buffer.from(uuid.replaceAll('-', ''), 'hex');
    const expected = Array.from({ length: 8 }, (_, index) => {
      const noun = nouns[bytes[index * 2 + 1]];
      return modifiers[bytes[index * 2]] + noun[0].toUpperCase() + noun.slice(1);
    }).join('-');
    const phrase = encode(uuid);
    assert.equal(phrase, expected);
    assert.equal(phrase.split('-').length, 8);
    assert.equal(decode(phrase), uuid);
    assert.equal(decode(phrase.toUpperCase()), uuid);
    assert.equal(decode(phrase.toLowerCase()), uuid);
  }
});

test('invalid options, UUIDs and phrases are rejected', () => {
  for (const options of [{ separator: '' }, { separator: 'word' }]) {
    assert.throws(() => encode(nil, options));
    assert.throws(() => decode('invalid', options));
  }
  assert.throws(() => encode('not a UUID'));
  assert.throws(() => encodeBatch([nil, 'bad']));
  assert.throws(() => decodeBatch([encode(nil), 'bad']));
  assert.deepEqual(encodeBatch([]), []);
  assert.deepEqual(decodeBatch([]), []);
  const phrase = encode(nil);
  for (const malformed of ['', `-${phrase}`, `${phrase}-`, `${phrase}-extra`, 'unknown']) {
    assert.throws(() => decode(malformed));
  }
});

test('UUID spelling canonicalization', () => {
  const uuid = '550e8400-e29b-41d4-a716-446655440000';
  assert.equal(encode(uuid), encode(uuid.toUpperCase()));
  assert.equal(encode(uuid), encode(uuid.replaceAll('-', '')));
  assert.equal(decode(encode(maximum)), maximum);
});
