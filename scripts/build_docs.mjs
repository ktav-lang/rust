#!/usr/bin/env node
// build_docs.mjs — assembles the trilingual markdown artifacts of this
// repository from per-meaning source units.
//
// Why this exists: README.md/.ru.md/.zh.md (and, once migrated, the
// CHANGELOG trio) used to be three independently hand-maintained files.
// Twice in one review cycle the English file was corrected while the
// Russian and Chinese ones kept the stale claim (findings R14-F3 and
// R15-F4). Generating all three from one source makes that class of
// defect structurally impossible rather than merely discouraged.
//
// Source model — a flat, ordered array of units, one unit per granular
// meaning (a paragraph, a rule, a bullet, a table row). A unit is
// EITHER a language triple or a language-independent block:
//
//   export default [
//     { id: "intro", en: `...`, ru: `...`, zh: `...` },
//     { id: "example", common: "```text\nname: Russia\n```" },
//   ];
//
// `common` is for content that is identical in every language — code
// samples, badges, Ktav snippets. Triplicating those would invite the
// very drift this tool prevents; one copy makes it impossible.
//
// Usage:
//   node scripts/build_docs.mjs           write every configured artifact
//   node scripts/build_docs.mjs --check   verify byte-identical, no writes
//   node scripts/build_docs.mjs -h        usage
//
// Node ESM, built-ins only, offline. Exits non-zero on any validation
// failure or, under --check, on the first byte difference.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

export const LANGS = ['en', 'ru', 'zh'];

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

// Each document: the source module, and the artifact each language
// lands in. Add a document here when it gains a source; never add an
// artifact that is still hand-maintained.
export const DOCUMENTS = [
  {
    name: 'README',
    source: 'docs/content/readme.units.mjs',
    outputs: { en: 'README.md', ru: 'README.ru.md', zh: 'README.zh.md' },
  },
];

class BuildError extends Error {}

function fail(msg) {
  throw new BuildError(msg);
}

// ---------------------------------------------------------------------------
// Validation
//
// Every rule here is a hard failure. A missing language must never be
// emitted as a silent gap — that is the whole point of the unit model.
// ---------------------------------------------------------------------------

export function validateUnits(docName, units) {
  if (!Array.isArray(units)) {
    fail(`${docName}: source default export must be an array of units`);
  }
  if (units.length === 0) {
    fail(`${docName}: source exports no units`);
  }

  const seen = new Set();

  units.forEach((unit, index) => {
    const at = `${docName}: unit #${index}`;

    if (unit === null || typeof unit !== 'object' || Array.isArray(unit)) {
      fail(`${at} is not an object`);
    }

    const { id } = unit;
    if (typeof id !== 'string' || id.trim() === '') {
      fail(`${at} has no non-empty string "id"`);
    }
    if (seen.has(id)) {
      fail(`${docName}: duplicate unit id "${id}"`);
    }
    seen.add(id);

    const hasCommon = Object.hasOwn(unit, 'common');
    const present = LANGS.filter((lang) => Object.hasOwn(unit, lang));

    if (hasCommon && present.length > 0) {
      fail(`${docName}: unit "${id}" mixes "common" with ${present.join('/')} — `
        + 'a unit is either language-independent or a full triple');
    }

    if (hasCommon) {
      if (typeof unit.common !== 'string' || unit.common.trim() === '') {
        fail(`${docName}: unit "${id}" has an empty "common"`);
      }
      return;
    }

    // A triple must be complete. Reporting the missing languages by name
    // matters: the usual mistake is adding English and forgetting the
    // other two, and the message should say exactly that.
    const missing = LANGS.filter((lang) => !Object.hasOwn(unit, lang));
    if (missing.length > 0) {
      fail(`${docName}: unit "${id}" is missing ${missing.join(', ')} `
        + `(a unit needs all of ${LANGS.join('/')}, or a single "common")`);
    }

    for (const lang of LANGS) {
      if (typeof unit[lang] !== 'string') {
        fail(`${docName}: unit "${id}" field "${lang}" is not a string`);
      }
      if (unit[lang].trim() === '') {
        fail(`${docName}: unit "${id}" field "${lang}" is empty — `
          + 'write the translation, or make the unit "common"');
      }
    }
  });

  return units;
}

// ---------------------------------------------------------------------------
// Assembly
// ---------------------------------------------------------------------------

export function renderLanguage(units, lang) {
  const blocks = units.map((unit) =>
    (Object.hasOwn(unit, 'common') ? unit.common : unit[lang]).replace(/\s+$/u, ''));
  return `${blocks.join('\n\n')}\n`;
}

async function loadUnits(doc) {
  const sourcePath = path.join(REPO_ROOT, doc.source);
  if (!fs.existsSync(sourcePath)) {
    fail(`${doc.name}: source ${doc.source} does not exist`);
  }
  const mod = await import(pathToFileURL(sourcePath).href);
  if (!mod || mod.default === undefined) {
    fail(`${doc.name}: ${doc.source} has no default export`);
  }
  return validateUnits(doc.name, mod.default);
}

export async function buildDocument(doc) {
  const units = await loadUnits(doc);
  const rendered = new Map();
  for (const lang of LANGS) {
    rendered.set(lang, renderLanguage(units, lang));
  }
  return rendered;
}

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

function firstDifference(existing, expected) {
  const limit = Math.min(existing.length, expected.length);
  for (let i = 0; i < limit; i += 1) {
    if (existing[i] !== expected[i]) return i;
  }
  return existing.length === expected.length ? -1 : limit;
}

function lineAt(buf, offset) {
  const upto = buf.subarray(0, offset).toString('utf8');
  const line = upto.split('\n').length;
  return line;
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

const USAGE = `build_docs.mjs — assemble trilingual markdown from source units

  node scripts/build_docs.mjs           write every configured artifact
  node scripts/build_docs.mjs --check   verify byte-identical, no writes
  node scripts/build_docs.mjs -h        this message
`;

async function main(argv) {
  if (argv.includes('-h') || argv.includes('--help')) {
    process.stdout.write(USAGE);
    return 0;
  }

  const check = argv.includes('--check');
  const unknown = argv.filter((a) => !['--check'].includes(a));
  if (unknown.length > 0) {
    process.stderr.write(`build_docs: unknown argument ${unknown[0]}\n\n${USAGE}`);
    return 2;
  }

  let stale = 0;
  let written = 0;

  for (const doc of DOCUMENTS) {
    const rendered = await buildDocument(doc);

    for (const lang of LANGS) {
      const outPath = path.join(REPO_ROOT, doc.outputs[lang]);
      const expected = Buffer.from(rendered.get(lang), 'utf8');

      if (!check) {
        fs.writeFileSync(outPath, expected);
        written += 1;
        continue;
      }

      if (!fs.existsSync(outPath)) {
        process.stderr.write(`build_docs: ${doc.outputs[lang]} is missing — run without --check\n`);
        stale += 1;
        continue;
      }

      const existing = fs.readFileSync(outPath);
      const at = firstDifference(existing, expected);
      if (at !== -1) {
        process.stderr.write(
          `build_docs: ${doc.outputs[lang]} differs from its source at byte ${at} `
          + `(line ${lineAt(expected, at)}). The artifact is generated: edit `
          + `${doc.source} and re-run "node scripts/build_docs.mjs".\n`);
        stale += 1;
      }
    }
  }

  if (check) {
    if (stale > 0) {
      process.stderr.write(`build_docs: ${stale} artifact(s) out of date\n`);
      return 1;
    }
    process.stdout.write(`build_docs: all artifacts byte-identical to their sources\n`);
    return 0;
  }

  process.stdout.write(`build_docs: wrote ${written} artifact(s)\n`);
  return 0;
}

const invokedDirectly = process.argv[1]
  && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);

if (invokedDirectly) {
  try {
    process.exitCode = await main(process.argv.slice(2));
  } catch (err) {
    if (err instanceof BuildError) {
      process.stderr.write(`build_docs: ${err.message}\n`);
      process.exitCode = 1;
    } else {
      throw err;
    }
  }
}
