import { readFileSync } from 'node:fs';
import { checkLicenses } from './npm-license-policy.mjs';

const lock = JSON.parse(readFileSync('package-lock.json', 'utf8'));
const failures = checkLicenses(lock);
for (const failure of failures) console.error(failure);
if (failures.length > 0) process.exitCode = 1;
