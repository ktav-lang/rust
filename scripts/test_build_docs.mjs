#!/usr/bin/env node
// Adversarial tests for build_docs.mjs. Run: node --test scripts/test_build_docs.mjs
//
// The builder's value is entirely in what it REFUSES. A generator that
// quietly emits a document with a missing translation is worse than no
// generator at all, because it launders the gap as machine output. Most
// of the tests below therefore drive a refusal, not a success.

import assert from 'node:assert/strict';
import test from 'node:test';

import { LANGS, renderLanguage, validateUnits } from './build_docs.mjs';

function rejects(units, fragment) {
  assert.throws(
    () => validateUnits('DOC', units),
    (err) => {
      assert.match(err.message, fragment);
      return true;
    },
    `expected rejection matching ${fragment}`,
  );
}

// --- refusals ---------------------------------------------------------------

test('a triple missing one language is rejected, naming that language', () => {
  // The archetypal mistake this tool exists to stop: English written,
  // the other two forgotten (review findings R14-F3, R15-F4).
  rejects([{ id: 'a', en: 'Text', ru: 'Текст' }], /missing zh/u);
  rejects([{ id: 'a', en: 'Text', zh: '文本' }], /missing ru/u);
  rejects([{ id: 'a', en: 'Text' }], /missing ru, zh/u);
});

test('a blank language is rejected as loudly as a missing one', () => {
  // Otherwise "" becomes a lawful way to skip a translation.
  rejects([{ id: 'a', en: 'Text', ru: '   ', zh: '文本' }], /field "ru" is empty/u);
});

test('mixing common with a language field is rejected', () => {
  rejects(
    [{ id: 'a', common: 'code', en: 'Text' }],
    /mixes "common" with en/u,
  );
});

test('an empty common block is rejected', () => {
  rejects([{ id: 'a', common: '  ' }], /has an empty "common"/u);
});

test('units require unique non-empty ids', () => {
  rejects([{ id: '', en: 'a', ru: 'b', zh: 'c' }], /no non-empty string "id"/u);
  rejects([{ en: 'a', ru: 'b', zh: 'c' }], /no non-empty string "id"/u);
  rejects(
    [
      { id: 'dup', en: 'a', ru: 'b', zh: 'c' },
      { id: 'dup', en: 'd', ru: 'e', zh: 'f' },
    ],
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

// --- rendering --------------------------------------------------------------

test('a complete triple is accepted', () => {
  const units = [{ id: 'a', en: 'Text', ru: 'Текст', zh: '文本' }];
  assert.equal(validateUnits('DOC', units), units);
});

test('renderLanguage joins units with one blank line and a trailing newline', () => {
  const units = [
    { id: 'a', en: '# Title', ru: '# Заголовок', zh: '# 标题' },
    { id: 'b', en: 'First paragraph.', ru: 'Первый абзац.', zh: '第一段。' },
  ];
  assert.equal(renderLanguage(units, 'en'), '# Title\n\nFirst paragraph.\n');
  assert.equal(renderLanguage(units, 'ru'), '# Заголовок\n\nПервый абзац.\n');
  assert.equal(renderLanguage(units, 'zh'), '# 标题\n\n第一段。\n');
});

test('a common unit renders byte-identically into every language', () => {
  const units = [
    { id: 'prose', en: 'Example:', ru: 'Пример:', zh: '示例:' },
    { id: 'code', common: '```text\nname: Russia\n```' },
  ];
  const rendered = LANGS.map((lang) => renderLanguage(units, lang));
  const codeOf = (s) => s.slice(s.indexOf('```'));
  // Byte-identical across languages is the reason `common` exists
  // instead of triplicating the snippet.
  assert.equal(codeOf(rendered[0]), codeOf(rendered[1]));
  assert.equal(codeOf(rendered[1]), codeOf(rendered[2]));
});

test('trailing whitespace inside a unit does not leak into the artifact', () => {
  const units = [{ id: 'a', en: 'Text.   \n\n', ru: 'Текст.  ', zh: '文本。 ' }];
  assert.equal(renderLanguage(units, 'en'), 'Text.\n');
  assert.equal(renderLanguage(units, 'ru'), 'Текст.\n');
});

test('units render in source order', () => {
  const units = [
    { id: 'one', en: 'One', ru: 'Один', zh: '一' },
    { id: 'two', en: 'Two', ru: 'Два', zh: '二' },
    { id: 'three', en: 'Three', ru: 'Три', zh: '三' },
  ];
  validateUnits('DOC', units);
  assert.equal(renderLanguage(units, 'en'), 'One\n\nTwo\n\nThree\n');
});
