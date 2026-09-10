#!/usr/bin/env node
// Tests for @ktav-lang/docs-gen. Run: node --test test
//
// The package's value is mostly in what it REFUSES, so most of these
// drive a refusal. A generator that quietly emits a document with a
// missing translation is worse than no generator, because it launders
// the gap as machine output.

import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  DocsGenError,
  checkDocuments,
  firstDifference,
  headingSkeleton,
  loadConfig,
  renderLanguage,
  structuralProblems,
  validateUnits,
  writeDocuments,
} from '../src/index.mjs';

const LANGS = ['en', 'ru', 'zh'];

function rejects(units, fragment, langs = LANGS) {
  assert.throws(
    () => validateUnits('DOC', units, langs),
    (err) => {
      assert.ok(err instanceof DocsGenError, `expected DocsGenError, got ${err.name}`);
      assert.match(err.message, fragment);
      return true;
    },
    `expected rejection matching ${fragment}`,
  );
}

// --- refusals ---------------------------------------------------------------

test('a translation set missing one language is rejected, naming it', () => {
  rejects([{ id: 'a', en: 'Text', ru: 'Текст' }], /missing zh/u);
  rejects([{ id: 'a', en: 'Text', zh: '文本' }], /missing ru/u);
  rejects([{ id: 'a', en: 'Text' }], /missing ru, zh/u);
});

test('a blank language is rejected as loudly as a missing one', () => {
  rejects([{ id: 'a', en: 'Text', ru: '   ', zh: '文本' }], /field "ru" is empty/u);
});

test('mixing common with a language field is rejected', () => {
  rejects([{ id: 'a', common: 'code', en: 'Text' }], /mixes "common" with en/u);
});

test('an empty common block is rejected', () => {
  rejects([{ id: 'a', common: '  ' }], /has an empty "common"/u);
});

test('units require unique non-empty ids', () => {
  rejects([{ id: '', en: 'a', ru: 'b', zh: 'c' }], /no non-empty string "id"/u);
  rejects([{ en: 'a', ru: 'b', zh: 'c' }], /no non-empty string "id"/u);
  rejects(
    [{ id: 'dup', en: 'a', ru: 'b', zh: 'c' }, { id: 'dup', en: 'd', ru: 'e', zh: 'f' }],
    /duplicate unit id "dup"/u,
  );
});

test('a non-array or empty source is rejected', () => {
  rejects({ id: 'a' }, /must be an array of units/u);
  rejects([], /exports no units/u);
  rejects(['not an object'], /is not an object/u);
});

test('a non-string language field is rejected', () => {
  rejects([{ id: 'a', en: 42, ru: 'b', zh: 'c' }], /field "en" is not a string/u);
});

// --- configurable language set ----------------------------------------------

test('the language set is configuration, not a constant', () => {
  // Two languages, neither of them English: the package must not assume
  // any particular set, or it cannot serve another repository.
  const units = [{ id: 'a', de: 'Text', fr: 'Texte' }];
  assert.equal(validateUnits('DOC', units, ['de', 'fr']), units);
  assert.equal(renderLanguage(units, 'fr'), 'Texte\n');

  // The same units are incomplete under a wider set.
  rejects(units, /missing es/u, ['de', 'fr', 'es']);
});

test('an empty language set is rejected', () => {
  rejects([{ id: 'a', en: 'x' }], /no languages configured/u, []);
});

test('language ids are opaque strings, including BCP-47 style tags', () => {
  // The library must not parse, normalise or validate the shape of a
  // language id — it is the caller's identifier, not a locale the
  // library understands.
  const langs = ['pt-BR', 'zh-Hans', 'x-pirate'];
  const units = [{ id: 'a', 'pt-BR': 'Olá', 'zh-Hans': '你好', 'x-pirate': 'Ahoy' }];
  assert.equal(validateUnits('DOC', units, langs), units);
  assert.equal(renderLanguage(units, 'x-pirate'), 'Ahoy\n');
  rejects([{ id: 'a', 'pt-BR': 'Olá' }], /missing zh-Hans, x-pirate/u, langs);
});

test('a single-language project is legitimate', () => {
  const units = [{ id: 'a', en: 'Only one language here' }];
  assert.equal(validateUnits('DOC', units, ['en']), units);
});

// --- configurable separator -------------------------------------------------

test('the unit separator is configurable for non-markdown output', () => {
  const units = [
    { id: 'a', en: 'one' },
    { id: 'b', en: 'two' },
  ];
  assert.equal(renderLanguage(units, 'en'), 'one\n\ntwo\n');          // markdown default
  assert.equal(renderLanguage(units, 'en', '\n'), 'one\ntwo\n');      // single newline
  assert.equal(renderLanguage(units, 'en', '\n---\n'), 'one\n---\ntwo\n');
});

// --- rendering --------------------------------------------------------------

test('renderLanguage joins units with one blank line and a trailing newline', () => {
  const units = [
    { id: 'a', en: '# Title', ru: '# Заголовок', zh: '# 标题' },
    { id: 'b', en: 'First paragraph.', ru: 'Первый абзац.', zh: '第一段。' },
  ];
  assert.equal(renderLanguage(units, 'en'), '# Title\n\nFirst paragraph.\n');
  assert.equal(renderLanguage(units, 'ru'), '# Заголовок\n\nПервый абзац.\n');
});

test('a common unit renders byte-identically into every language', () => {
  const units = [
    { id: 'prose', en: 'Example:', ru: 'Пример:', zh: '示例:' },
    { id: 'code', common: '```text\nname: Russia\n```' },
  ];
  const rendered = LANGS.map((lang) => renderLanguage(units, lang));
  const codeOf = (s) => s.slice(s.indexOf('```'));
  assert.equal(codeOf(rendered[0]), codeOf(rendered[1]));
  assert.equal(codeOf(rendered[1]), codeOf(rendered[2]));
});

test('trailing whitespace inside a unit does not leak into the artifact', () => {
  const units = [{ id: 'a', en: 'Text.   \n\n', ru: 'Текст.  ', zh: '文本。 ' }];
  assert.equal(renderLanguage(units, 'en'), 'Text.\n');
});

test('firstDifference reports the first differing byte, or -1', () => {
  assert.equal(firstDifference(Buffer.from('abc'), Buffer.from('abc')), -1);
  assert.equal(firstDifference(Buffer.from('abc'), Buffer.from('abd')), 2);
  // A pure truncation differs at the point where the shorter one ends.
  assert.equal(firstDifference(Buffer.from('ab'), Buffer.from('abc')), 2);
});

// --- tight joining ----------------------------------------------------------

test('tight units make a list item a unit of its own', () => {
  // Under the blank-line separator one bullet per unit renders as a
  // loose list; `join: "tight"` is what makes per-bullet granularity
  // possible at all.
  const units = [
    { id: 'h', en: '### Rules' },
    { id: 'b1', en: '- first' },
    { id: 'b2', en: '- second', join: 'tight' },
    { id: 'b3', en: '- third', join: 'tight' },
  ];
  validateUnits('DOC', units, ['en']);
  assert.equal(renderLanguage(units, 'en'), '### Rules\n\n- first\n- second\n- third\n');
});

test('tight units keep a table intact one row at a time', () => {
  const units = [
    { id: 'head', common: '| a | b |' },
    { id: 'sep', common: '|---|---|', join: 'tight' },
    { id: 'r1', en: '| one | two |', join: 'tight' },
  ];
  validateUnits('DOC', units, ['en']);
  assert.equal(renderLanguage(units, 'en'), '| a | b |\n|---|---|\n| one | two |\n');
});

test('the first unit cannot be tight, and join values are checked', () => {
  rejects([{ id: 'a', en: 'x', join: 'tight' }], /first unit .* cannot be join/u, ['en']);
  rejects(
    [{ id: 'a', en: 'x' }, { id: 'b', en: 'y', join: 'snug' }],
    /has join "snug"/u,
    ['en'],
  );
});

// --- structural parity ------------------------------------------------------

test('headingSkeleton ignores headings inside fenced code', () => {
  // This project's own documents embed Ktav samples whose `##` comment
  // lines would otherwise be counted as document headings.
  const md = [
    '# Title',
    '',
    '```text',
    '## not a heading, it is a Ktav comment',
    '### neither is this',
    '```',
    '',
    '## Real section',
    '',
    '~~~',
    '# also fenced',
    '~~~',
  ].join('\n');
  assert.deepEqual(headingSkeleton(md), [1, 2]);
});

test('headingSkeleton closes a fence only on its own marker', () => {
  const md = ['```', '~~~', '# still inside the backtick fence', '```', '# real'].join('\n');
  assert.deepEqual(headingSkeleton(md), [1]);
});

test('structural parity catches a demoted heading', () => {
  const rendered = new Map([
    ['en', '# T\n\n## Section\n'],
    ['ru', '# T\n\n### Раздел\n'],
  ]);
  const problems = structuralProblems('DOC', rendered, ['en', 'ru']);
  assert.equal(problems.length, 1);
  assert.match(problems[0], /heading #2 is level 3 in ru but level 2 in en/u);
});

test('structural parity catches a missing or extra heading', () => {
  const rendered = new Map([
    ['en', '# T\n\n## A\n\n## B\n'],
    ['ru', '# T\n\n## A\n'],
  ]);
  const problems = structuralProblems('DOC', rendered, ['en', 'ru']);
  assert.equal(problems.length, 1);
  assert.match(problems[0], /ru has 2 heading\(s\) but en has 3/u);
});

test('structural parity is silent when the languages agree', () => {
  const rendered = new Map([
    ['en', '# T\n\n## A\n'],
    ['ru', '# Т\n\n## А\n'],
  ]);
  assert.deepEqual(structuralProblems('DOC', rendered, ['en', 'ru']), []);
});

// --- end to end -------------------------------------------------------------

function scratchRepo() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'docs-gen-'));
  fs.mkdirSync(path.join(dir, 'src'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'src', 'units.mjs'),
    "export default [{ id: 'a', en: 'Hello', de: 'Hallo' }];\n");
  fs.writeFileSync(path.join(dir, 'docs.config.mjs'),
    "export default { languages: ['en', 'de'], documents: [{ name: 'DOC',"
    + " source: 'src/units.mjs', outputs: { en: 'DOC.md', de: 'DOC.de.md' } }] };\n");
  return dir;
}

test('write then check round-trips, and a hand edit is caught', async () => {
  const dir = scratchRepo();
  try {
    const config = await loadConfig(path.join(dir, 'docs.config.mjs'));
    assert.deepEqual(config.languages, ['en', 'de']);

    const { written } = await writeDocuments(config);
    assert.equal(written.length, 2);
    assert.equal(fs.readFileSync(path.join(dir, 'DOC.md'), 'utf8'), 'Hello\n');
    assert.equal(fs.readFileSync(path.join(dir, 'DOC.de.md'), 'utf8'), 'Hallo\n');

    assert.deepEqual((await checkDocuments(config)).stale, []);

    // The gate must not be vacuous.
    fs.appendFileSync(path.join(dir, 'DOC.de.md'), 'stray\n');
    const { stale } = await checkDocuments(config);
    assert.equal(stale.length, 1);
    assert.equal(stale[0].file, 'DOC.de.md');
    assert.match(stale[0].reason, /differs at byte/u);

    // A deleted artifact is stale too, not silently regenerated by check.
    fs.rmSync(path.join(dir, 'DOC.md'));
    const after = await checkDocuments(config);
    assert.ok(after.stale.some((s) => s.file === 'DOC.md' && s.reason === 'missing'));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('a malformed config is rejected with a reason', async () => {
  const dir = scratchRepo();
  // One config FILE per case on purpose: Node caches ES modules by URL,
  // so rewriting a single path and re-importing it silently re-runs the
  // first version and makes every later assertion meaningless. This test
  // caught exactly that.
  const cases = [
    ['empty-langs', 'export default { languages: [], documents: [] };',
      /"languages" must be a non-empty array/u],
    ['dup-langs', "export default { languages: ['en', 'en'], documents: [] };",
      /contains duplicates/u],
    // "id" and "common" are structural keys of a unit; allowing them as
    // language names would make a unit ambiguous.
    ['reserved-lang', "export default { languages: ['en', 'id'], documents: [] };",
      /reserved and cannot be languages/u],
    ['no-docs', "export default { languages: ['en'], documents: [] };",
      /"documents" must be a non-empty array/u],
    ['no-source', "export default { languages: ['en'], documents: [{ name: 'D' }] };",
      /has no "source"/u],
  ];
  try {
    for (const [name, body, expected] of cases) {
      const file = path.join(dir, `${name}.config.mjs`);
      fs.writeFileSync(file, `${body}\n`);
      await assert.rejects(() => loadConfig(file), expected, `case ${name}`);
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('config carries the separator through to the artifacts', async () => {
  const dir = scratchRepo();
  try {
    fs.writeFileSync(path.join(dir, 'src', 'units.mjs'),
      "export default [{ id: 'a', en: 'one' }, { id: 'b', en: 'two' }];\n");
    fs.writeFileSync(path.join(dir, 'sep.config.mjs'),
      "export default { languages: ['en'], separator: '\\n', documents: [{ name: 'D',"
      + " source: 'src/units.mjs', outputs: { en: 'D.md' } }] };\n");
    const config = await loadConfig(path.join(dir, 'sep.config.mjs'));
    await writeDocuments(config);
    assert.equal(fs.readFileSync(path.join(dir, 'D.md'), 'utf8'), 'one\ntwo\n');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('a non-string separator is rejected', async () => {
  const dir = scratchRepo();
  try {
    const file = path.join(dir, 'badsep.config.mjs');
    fs.writeFileSync(file,
      "export default { languages: ['en'], separator: 5, documents: [{ name: 'D',"
      + " source: 'src/units.mjs', outputs: { en: 'D.md' } }] };\n");
    await assert.rejects(() => loadConfig(file), /"separator" must be a string/u);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('rootDir defaults to the config directory and is overridable', async () => {
  const dir = scratchRepo();
  try {
    // Default: sources and outputs resolve next to the config.
    const config = await loadConfig(path.join(dir, 'docs.config.mjs'));
    assert.equal(config.rootDir, fs.realpathSync(dir));

    // Explicit rootDir is resolved relative to the config file, so a
    // config may live outside the tree it describes.
    const nested = path.join(dir, 'nested');
    fs.mkdirSync(path.join(nested, 'src'), { recursive: true });
    fs.writeFileSync(path.join(nested, 'src', 'units.mjs'),
      "export default [{ id: 'a', en: 'Nested' }];\n");
    fs.writeFileSync(path.join(dir, 'root.config.mjs'),
      "export default { rootDir: 'nested', languages: ['en'], documents: [{ name: 'D',"
      + " source: 'src/units.mjs', outputs: { en: 'D.md' } }] };\n");
    const rooted = await loadConfig(path.join(dir, 'root.config.mjs'));
    await writeDocuments(rooted);
    assert.equal(fs.readFileSync(path.join(nested, 'D.md'), 'utf8'), 'Nested\n');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('a document missing an output for a configured language is rejected', async () => {
  const dir = scratchRepo();
  try {
    fs.writeFileSync(path.join(dir, 'docs.config.mjs'),
      "export default { languages: ['en', 'de'], documents: [{ name: 'DOC',"
      + " source: 'src/units.mjs', outputs: { en: 'DOC.md' } }] };\n");
    const config = await loadConfig(path.join(dir, 'docs.config.mjs'));
    await assert.rejects(() => writeDocuments(config),
      /no output path configured for language "de"/u);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});
