import assert from 'node:assert/strict';
import test from 'node:test';
import { checkLicenses } from '../scripts/npm-license-policy.mjs';

function lock(packages) {
  return { lockfileVersion: 3, packages: { '': {}, ...packages } };
}

test('accepts explicitly approved licenses on nested dependencies', () => {
  assert.deepEqual(checkLicenses(lock({
    'node_modules/a': { license: 'MIT OR Apache-2.0' },
    'node_modules/a/node_modules/b': { license: 'ISC' },
  })), []);
});

test('rejects missing, unknown, and unapproved compound licenses', () => {
  const failures = checkLicenses(lock({
    'node_modules/a': {},
    'node_modules/b': { license: 'UNKNOWN' },
    'node_modules/c': { license: 'MIT AND GPL-3.0-only' },
    'node_modules/d': { license: 'MIT-fake' },
  }));
  assert.equal(failures.length, 4);
  for (const name of ['a', 'b', 'c', 'd']) {
    assert.ok(failures.some((failure) => failure.startsWith(`node_modules/${name}:`)));
  }
});

test('rejects missing or incompatible lockfile package maps', () => {
  assert.throws(() => checkLicenses({ lockfileVersion: 2, packages: {} }));
  assert.throws(() => checkLicenses({ lockfileVersion: 3 }));
  assert.throws(() => checkLicenses({ lockfileVersion: 3, packages: [] }));
});
