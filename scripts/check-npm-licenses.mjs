import { readFileSync } from 'node:fs';

const lock = JSON.parse(readFileSync('package-lock.json', 'utf8'));
if (lock.lockfileVersion !== 3 || !lock.packages) {
  throw new Error('Expected a package-lock.json with lockfileVersion 3 and packages');
}

const allowed = new Set([
  'Apache-2.0',
  'BSD-2-Clause',
  'BSD-3-Clause',
  'BSL-1.0',
  'ISC',
  'MIT',
  'Unicode-3.0',
  'Zlib',
  'MIT OR Apache-2.0',
  'Apache-2.0 OR MIT',
  '(MIT OR Apache-2.0)',
  '(Apache-2.0 OR MIT)',
]);

let failed = false;
for (const [path, pkg] of Object.entries(lock.packages)) {
  if (path === '') continue;
  if (typeof pkg.license !== 'string' || !allowed.has(pkg.license)) {
    console.error(`${path}: unapproved or missing license ${JSON.stringify(pkg.license)}`);
    failed = true;
  }
}
if (failed) process.exitCode = 1;
