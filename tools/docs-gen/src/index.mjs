// @ktav-lang/docs-gen — generate parallel multilingual markdown from
// per-meaning source units.
//
// The problem it solves: a project that ships the same document in
// several languages usually keeps one file per language. Nothing then
// ties a claim in one language to the same claim in the others, so a
// correction lands in one file and quietly rots in the rest. Reviews of
// this project found that exact defect twice in one cycle.
//
// The model: a document is a flat, ordered array of UNITS, and a unit is
// ONE granular meaning — a paragraph, a rule, a bullet, a table row.
// Not a sentence, and never a whole section: the point is that the
// languages of a single meaning sit together, so an edit to one of them
// is visibly missing its siblings.
//
// A unit is either a complete translation set:
//
//   { id: 'intro', en: '...', ru: '...', zh: '...' }
//
// or a language-independent block, written once:
//
//   { id: 'example', common: '```text\nname: Russia\n```' }
//
// `common` exists for content that is identical in every language — code
// samples, badges, tables of symbols. Repeating those per language would
// invite the very drift this module prevents; one copy makes it
// impossible rather than merely detectable.
//
// Nothing here knows about any particular language set, document or
// repository layout: languages and documents are configuration. That is
// what makes it reusable across repositories.

import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

/** Raised for every rejected source; carries a human-readable reason. */
export class DocsGenError extends Error {
  constructor(message) {
    super(message);
    this.name = 'DocsGenError';
  }
}

function fail(msg) {
  throw new DocsGenError(msg);
}

// ---------------------------------------------------------------------------
// Validation
//
// Every rule below is a hard failure. A generator that quietly emits a
// document with a missing translation is worse than no generator at all,
// because it launders the gap as machine output.
// ---------------------------------------------------------------------------

/**
 * Validate a unit array against a language set.
 *
 * @param {string} label     document name, used in messages
 * @param {unknown} units    the source's default export
 * @param {string[]} langs   e.g. ['en', 'ru', 'zh']
 * @returns {object[]} the same array, once proven well-formed
 */
export function validateUnits(label, units, langs) {
  if (!Array.isArray(langs) || langs.length === 0) {
    fail(`${label}: no languages configured`);
  }
  if (!Array.isArray(units)) {
    fail(`${label}: source default export must be an array of units`);
  }
  if (units.length === 0) {
    fail(`${label}: source exports no units`);
  }

  const seen = new Set();

  units.forEach((unit, index) => {
    const at = `${label}: unit #${index}`;

    if (unit === null || typeof unit !== 'object' || Array.isArray(unit)) {
      fail(`${at} is not an object`);
    }

    const { id } = unit;
    if (typeof id !== 'string' || id.trim() === '') {
      fail(`${at} has no non-empty string "id"`);
    }
    if (seen.has(id)) {
      fail(`${label}: duplicate unit id "${id}"`);
    }
    seen.add(id);

    const hasCommon = Object.hasOwn(unit, 'common');
    const present = langs.filter((lang) => Object.hasOwn(unit, lang));

    if (hasCommon && present.length > 0) {
      fail(`${label}: unit "${id}" mixes "common" with ${present.join('/')} — `
        + 'a unit is either language-independent or a full translation set');
    }

    if (hasCommon) {
      if (typeof unit.common !== 'string' || unit.common.trim() === '') {
        fail(`${label}: unit "${id}" has an empty "common"`);
      }
      return;
    }

    // Naming the missing languages matters: the usual mistake is writing
    // the first language and forgetting the rest, and the message should
    // say exactly which ones are absent.
    const missing = langs.filter((lang) => !Object.hasOwn(unit, lang));
    if (missing.length > 0) {
      fail(`${label}: unit "${id}" is missing ${missing.join(', ')} `
        + `(a unit needs all of ${langs.join('/')}, or a single "common")`);
    }

    for (const lang of langs) {
      if (typeof unit[lang] !== 'string') {
        fail(`${label}: unit "${id}" field "${lang}" is not a string`);
      }
      if (unit[lang].trim() === '') {
        fail(`${label}: unit "${id}" field "${lang}" is empty — `
          + 'write the translation, or make the unit "common"');
      }
    }
  });

  return units;
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/** Markdown's block separator; overridable for other output formats. */
export const DEFAULT_SEPARATOR = '\n\n';

/**
 * Render one language of a validated unit array.
 *
 * Trailing whitespace inside a unit is stripped, so a stray space cannot
 * change the artifact. Units are joined by `separator` — a blank line by
 * default, which is what markdown wants; a project emitting another
 * format configures its own. The result ends with exactly one newline.
 */
export function renderLanguage(units, lang, separator = DEFAULT_SEPARATOR) {
  const blocks = units.map((unit) =>
    (Object.hasOwn(unit, 'common') ? unit.common : unit[lang]).replace(/\s+$/u, ''));
  return `${blocks.join(separator)}\n`;
}

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

/**
 * @typedef {object} DocumentSpec
 * @property {string} name              label used in diagnostics
 * @property {string} source            path to an ES module default-exporting units
 * @property {Record<string,string>} outputs  language -> artifact path
 */

async function loadUnits(doc, langs, rootDir) {
  const sourcePath = path.resolve(rootDir, doc.source);
  if (!fs.existsSync(sourcePath)) {
    fail(`${doc.name}: source ${doc.source} does not exist`);
  }
  const mod = await import(pathToFileURL(sourcePath).href);
  if (!mod || mod.default === undefined) {
    fail(`${doc.name}: ${doc.source} has no default export`);
  }
  return validateUnits(doc.name, mod.default, langs);
}

/**
 * Build every language of one document.
 *
 * @returns {Promise<Map<string,string>>} language -> rendered markdown
 */
export async function buildDocument(doc, langs, rootDir, separator = DEFAULT_SEPARATOR) {
  for (const lang of langs) {
    if (!doc.outputs || typeof doc.outputs[lang] !== 'string') {
      fail(`${doc.name}: no output path configured for language "${lang}"`);
    }
  }
  const units = await loadUnits(doc, langs, rootDir);
  const rendered = new Map();
  for (const lang of langs) {
    rendered.set(lang, renderLanguage(units, lang, separator));
  }
  return rendered;
}

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

export function firstDifference(existing, expected) {
  const limit = Math.min(existing.length, expected.length);
  for (let i = 0; i < limit; i += 1) {
    if (existing[i] !== expected[i]) return i;
  }
  return existing.length === expected.length ? -1 : limit;
}

export function lineAtByte(buf, offset) {
  return buf.subarray(0, offset).toString('utf8').split('\n').length;
}

// ---------------------------------------------------------------------------
// Write / check
// ---------------------------------------------------------------------------

/**
 * Regenerate every configured artifact.
 *
 * @returns {Promise<{written: string[]}>}
 */
export async function writeDocuments(config) {
  const { rootDir, languages, documents, separator } = config;
  const written = [];
  for (const doc of documents) {
    const rendered = await buildDocument(doc, languages, rootDir, separator);
    for (const lang of languages) {
      const outPath = path.resolve(rootDir, doc.outputs[lang]);
      fs.mkdirSync(path.dirname(outPath), { recursive: true });
      fs.writeFileSync(outPath, Buffer.from(rendered.get(lang), 'utf8'));
      written.push(doc.outputs[lang]);
    }
  }
  return { written };
}

/**
 * Verify every artifact is byte-identical to what its source renders,
 * without writing anything.
 *
 * @returns {Promise<{stale: {file: string, source: string, reason: string}[]}>}
 */
export async function checkDocuments(config) {
  const { rootDir, languages, documents, separator } = config;
  const stale = [];
  for (const doc of documents) {
    const rendered = await buildDocument(doc, languages, rootDir, separator);
    for (const lang of languages) {
      const file = doc.outputs[lang];
      const outPath = path.resolve(rootDir, file);
      const expected = Buffer.from(rendered.get(lang), 'utf8');

      if (!fs.existsSync(outPath)) {
        stale.push({ file, source: doc.source, reason: 'missing' });
        continue;
      }

      const existing = fs.readFileSync(outPath);
      const at = firstDifference(existing, expected);
      if (at !== -1) {
        stale.push({
          file,
          source: doc.source,
          reason: `differs at byte ${at} (line ${lineAtByte(expected, at)})`,
        });
      }
    }
  }
  return { stale };
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/**
 * Load and validate a config module. The config is what makes this
 * package reusable: the library itself knows no languages, no paths and
 * no document names.
 */
export async function loadConfig(configPath) {
  const resolved = path.resolve(configPath);
  if (!fs.existsSync(resolved)) {
    fail(`config ${configPath} does not exist`);
  }
  const mod = await import(pathToFileURL(resolved).href);
  const config = mod.default;
  if (!config || typeof config !== 'object') {
    fail(`config ${configPath} has no default-exported object`);
  }

  const languages = config.languages;
  if (!Array.isArray(languages) || languages.length === 0
      || !languages.every((l) => typeof l === 'string' && l.trim() !== '')) {
    fail(`config ${configPath}: "languages" must be a non-empty array of strings`);
  }
  if (new Set(languages).size !== languages.length) {
    fail(`config ${configPath}: "languages" contains duplicates`);
  }
  if (languages.includes('id') || languages.includes('common')) {
    fail(`config ${configPath}: "id" and "common" are reserved and cannot be languages`);
  }

  const documents = config.documents;
  if (!Array.isArray(documents) || documents.length === 0) {
    fail(`config ${configPath}: "documents" must be a non-empty array`);
  }
  documents.forEach((doc, i) => {
    if (!doc || typeof doc.name !== 'string' || doc.name.trim() === '') {
      fail(`config ${configPath}: document #${i} has no "name"`);
    }
    if (typeof doc.source !== 'string' || doc.source.trim() === '') {
      fail(`config ${configPath}: document "${doc.name}" has no "source"`);
    }
    if (!doc.outputs || typeof doc.outputs !== 'object') {
      fail(`config ${configPath}: document "${doc.name}" has no "outputs" map`);
    }
  });

  if (config.separator !== undefined && typeof config.separator !== 'string') {
    fail(`config ${configPath}: "separator" must be a string`);
  }

  return {
    separator: config.separator ?? DEFAULT_SEPARATOR,
    rootDir: config.rootDir
      ? path.resolve(path.dirname(resolved), config.rootDir)
      : path.dirname(resolved),
    languages,
    documents,
  };
}
