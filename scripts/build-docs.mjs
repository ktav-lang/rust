#!/usr/bin/env node
// Rebuilds this repository's README/CHANGELOG from root-docs/, using
// @ktav-lang/polydoc. Run with --check for a CI-friendly, read-only
// verification instead of regenerating the files.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { configure, buildRootDocs, writeRootDocs, checkRootDocs } from '@ktav-lang/polydoc';

const LANGS = ['en', 'ru', 'zh'];
configure({
  langs: LANGS,
  rootDocuments: ['README', 'CHANGELOG', 'CONTRIBUTING', 'SECURITY'],
});

// SECURITY.md's supported-version line uses @@MINOR_LINE@@ so it can
// never quietly fall behind Cargo.toml's actual version the way a
// hand-written "0.1.x" once did.
function readCrateVersion(root) {
  const toml = fs.readFileSync(path.join(root, 'Cargo.toml'), 'utf8');
  const match = toml.match(/^version\s*=\s*"([^"]+)"/mu);
  if (!match) throw new Error('Cargo.toml: no top-level version field found');
  return match[1];
}

// Per-unit validation proves every meaning has every language. It does
// NOT prove the languages describe the same DOCUMENT: a heading demoted
// from ## to ### in one translation, or an extra heading in another,
// passes unit validation untouched. This check (originally from this
// repo's own pre-polydoc docs-gen tool) is what catches that class of
// drift.
function headingSkeleton(markdown) {
  const levels = [];
  let fenceChar = null;
  let fenceLen = 0;
  for (const line of markdown.split('\n')) {
    const fence = line.match(/^\s{0,3}(`{3,}|~{3,})/u);
    if (fence) {
      const char = fence[1][0];
      const len = fence[1].length;
      if (fenceChar === null) { fenceChar = char; fenceLen = len; }
      else if (char === fenceChar && len >= fenceLen) { fenceChar = null; }
      continue;
    }
    if (fenceChar !== null) continue;
    const heading = line.match(/^(#{1,6})\s+\S/u);
    if (heading) levels.push(heading[1].length);
  }
  return levels;
}

function structuralProblems(label, perLang) {
  const [reference, ...others] = LANGS;
  const base = headingSkeleton(perLang.get(reference).toString('utf8'));
  const problems = [];
  for (const lang of others) {
    const other = headingSkeleton(perLang.get(lang).toString('utf8'));
    if (other.length !== base.length) {
      problems.push(`${label}: ${lang} has ${other.length} heading(s) but ${reference} has ` +
        `${base.length} — the translations describe different documents`);
      continue;
    }
    const at = base.findIndex((level, i) => level !== other[i]);
    if (at !== -1) {
      problems.push(`${label}: heading #${at + 1} is level ${other[at]} in ${lang} but ` +
        `level ${base[at]} in ${reference}`);
    }
  }
  return problems;
}

function usage() {
  process.stderr.write(
    'usage: node scripts/build-docs.mjs [--check]\n' +
    '  (no args)  regenerate README.md/.ru.md/.zh.md and CHANGELOG.md/.ru.md/.zh.md\n' +
    '  --check    verify the generated files match root-docs/ without writing\n'
  );
}

function cli() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, '..');

  const args = process.argv.slice(2);
  if (args.includes('-h') || args.includes('--help')) { usage(); process.exit(0); }
  if (args.length > 1 || (args.length === 1 && args[0] !== '--check')) {
    usage();
    process.exit(1);
  }
  const checkMode = args[0] === '--check';

  let docs;
  try {
    const version = readCrateVersion(root);
    docs = buildRootDocs(root, { release: { version, released: '' } });
    for (const [name, perLang] of docs) {
      const problems = structuralProblems(name, perLang);
      if (problems.length > 0) {
        throw new Error(problems.join('\n'));
      }
    }
  } catch (e) {
    process.stderr.write(`build-docs: ${e.message}\n`);
    process.exit(1);
  }

  if (!checkMode) {
    writeRootDocs(root, docs);
    process.stdout.write(`build-docs: assembled ${docs.size} document(s) from root-docs/\n`);
    process.exit(0);
  }

  const problems = checkRootDocs(root, docs);
  if (problems.length > 0) {
    for (const problem of problems) {
      process.stderr.write(`build-docs --check: ${problem}\n`);
    }
    process.exit(1);
  }
  process.exit(0);
}

const isMain = process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;
if (isMain) cli();
