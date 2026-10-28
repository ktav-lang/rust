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

export function checkLicenses(lock) {
  if (lock?.lockfileVersion !== 3 || !lock.packages || typeof lock.packages !== 'object' ||
      Array.isArray(lock.packages) || !lock.packages['']) {
    throw new Error('Expected a package-lock.json with lockfileVersion 3 and packages');
  }

  const failures = [];
  for (const [path, pkg] of Object.entries(lock.packages)) {
    if (path === '') continue;
    if (typeof pkg?.license !== 'string' || !allowed.has(pkg.license)) {
      failures.push(`${path}: unapproved or missing license ${JSON.stringify(pkg?.license)}`);
    }
  }
  return failures;
}
