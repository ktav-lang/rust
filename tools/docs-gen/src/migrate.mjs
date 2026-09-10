// Migration helper: derives per-meaning source units from a document's
// existing markdown artifacts. It runs in the opposite direction to
// docs-gen — artifacts back to source — and is only ever used once per
// document, to bring a hand-maintained trio under the generator, or to
// re-derive units at a finer granularity than they were first written.
//
//   node tools/docs-gen/src/migrate.mjs CHANGELOG         # report alignment
//   node tools/docs-gen/src/migrate.mjs CHANGELOG --why   # explain misfits
//   node tools/docs-gen/src/migrate.mjs CHANGELOG --emit  # write the units
//
// The document name and its artifact paths come from docs.config.mjs, so
// the remaining hand-maintained trios (CONTRIBUTING, SECURITY) need no
// change here beyond being declared there.

import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const name = process.argv[2];
if (!name || name.startsWith('--')) {
  console.error('usage: node tools/docs-gen/src/migrate.mjs <DocumentName> [--why] [--emit]');
  process.exit(2);
}

const config = (await import(pathToFileURL('docs.config.mjs').href)).default;
const doc = config.documents.find((entry) => entry.name === name);
if (!doc) {
  console.error(`docs.config.mjs declares no document named "${name}" `
    + `(it has ${config.documents.map((d) => d.name).join(', ')})`);
  process.exit(2);
}
const LANGS = doc.outputs;

// ---------------------------------------------------------------------------
// Blocks: runs of non-blank lines, with fenced code kept whole.
// ---------------------------------------------------------------------------

// Blocks carry the whitespace that preceded them, because "blank line
// between" and "nothing between" are different documents: two fenced
// samples written back to back are a single visual pair, and inserting a
// blank line between them would be a real edit.
function blocks(text) {
  const out = [];
  let current = [];
  let fence = null;
  let blankBefore = false;

  const flush = () => {
    if (current.length > 0) {
      out.push({ text: current.join('\n'), glue: blankBefore ? '\n\n' : '\n' });
      blankBefore = false;
    }
    current = [];
  };

  for (const line of text.split('\n')) {
    if (fence) {
      current.push(line);
      if (line.startsWith(fence)) fence = null;
      continue;
    }
    const opener = line.match(/^(```|~~~)/);
    if (opener) {
      flush();
      fence = opener[1];
      current.push(line);
      continue;
    }
    if (line.trim() === '') {
      flush();
      blankBefore = true;
    } else current.push(line);
  }
  flush();
  return out;
}

// A block that is a top-level bullet list becomes one atom per bullet;
// continuation and nested lines stay with the bullet that owns them.
const ITEM = /^(?:- |\d+\. )/;

function bullets(block) {
  if (block.startsWith('```') || !ITEM.test(block.split('\n')[0] ?? '')) {
    if (!block.split('\n').some((line) => ITEM.test(line))) return [block];
  }
  const items = [];
  for (const line of block.split('\n')) {
    if (ITEM.test(line) || items.length === 0) items.push([line]);
    else items.at(-1).push(line);
  }
  return items.map((item) => item.join('\n'));
}

// ---------------------------------------------------------------------------
// Sentences: the granularity that actually matters. A unit should be one
// self-contained claim, not a paragraph — a reviewer has to be able to
// see that the English moved while its translations did not.
// ---------------------------------------------------------------------------

// Abbreviations and ordinals whose full stop does not end a sentence.
const NOT_A_STOP = /(?:\b(?:e\.g|i\.e|cf|vs|etc|Mr|Fig|no)|\bт\.\s?е|\bт\.\s?д|\bсм|\d)$/u;

const LATIN_BOUNDARY = /([.!?])([ \n]+)(?=[A-ZА-ЯЁ«"`*(\[0-9])/gu;

// A CJK full stop is only a sentence end when real text follows it. The
// translations write `。` before a nested bullet and before the `**` that
// closes a bold run, and neither is a boundary — the English writes the
// very same shapes with the stop inside the markup.
const CJK_BOUNDARY = /([。！？])([ \n]*)(?=[^\s)\]}）」』】、。！？,-])/gu;

/**
 * Split one atom into sentences, returning `[{ text, glue }]` where
 * `glue` is the whitespace that preceded the sentence in the artifact.
 * Never splits inside a code span, a fenced block, a table, or a
 * heading.
 */
function sentences(atom, lang) {
  if (/^(```|\||#{1,6} |    |      )/.test(atom)) return [{ text: atom, glue: '' }];

  const pattern = lang === 'zh' ? CJK_BOUNDARY : LATIN_BOUNDARY;
  pattern.lastIndex = 0;

  const cuts = [];
  let match = pattern.exec(atom);
  while (match !== null) {
    const before = atom.slice(0, match.index + 1);
    // A stop inside an unclosed code span is punctuation in code, not a
    // sentence end: `Span::line_col(input)` and friends are full of them.
    const openCodeSpan = (before.match(/`/gu) || []).length % 2 === 1;
    // Likewise a stop inside an unclosed bold run: every entry opens with
    // `- **Headline.**`, and the `**` that follows the stop closes it
    // rather than starting the next sentence.
    const openBold = (before.match(/\*\*/gu) || []).length % 2 === 1
      && atom.slice(match.index + 1 + match[2].length).startsWith('*');
    if (!openCodeSpan && !openBold && !NOT_A_STOP.test(atom.slice(0, match.index))) {
      // "newline + indent" is a line break plus the continuation indent
      // of the enclosing list item. Only the break is glue; the indent
      // belongs to the sentence that follows it.
      const glue = /^\n[ \t]+$/.test(match[2]) ? '\n' : match[2];
      cuts.push({ at: match.index + 1, glue, consume: glue.length });
    }
    match = pattern.exec(atom);
  }
  if (cuts.length === 0) return [{ text: atom, glue: '' }];

  const parts = [];
  let start = 0;
  let glue = '';
  for (const cut of cuts) {
    parts.push({ text: atom.slice(start, cut.at), glue });
    start = cut.at + cut.consume;
    glue = cut.glue;
  }
  parts.push({ text: atom.slice(start), glue });
  return parts;
}

const GLUE_MODE = {
  '': 'none', ' ': 'flow', '\n': 'tight', '\n\n': 'block',
};

function modeFor(glue) {
  if (Object.hasOwn(GLUE_MODE, glue)) return GLUE_MODE[glue];
  // A boundary the artifact wrote as "newline + indent": the indent
  // belongs to the sentence that follows, so only the break is glue.
  if (/^\n[ \t]*$/.test(glue)) return 'tight';
  return null;
}

// ---------------------------------------------------------------------------
// Alignment
// ---------------------------------------------------------------------------

// Atoms carry the whitespace that separated them in the artifact: a
// blank line between blocks, a bare newline between the items of one
// list. Losing that distinction would turn every tight list into a
// loose one.
function atoms(text) {
  const out = [];
  for (const block of blocks(text)) {
    bullets(block.text).forEach((item, index) => {
      out.push({ text: item, glue: index === 0 ? block.glue : '\n' });
    });
  }
  if (out.length > 0) out[0].glue = '';
  return out;
}

const langs = Object.keys(LANGS);
const parsed = Object.fromEntries(
  Object.entries(LANGS).map(([lang, file]) => [lang, atoms(readFileSync(file, 'utf8'))]),
);

const counts = langs.map((lang) => parsed[lang].length);
if (new Set(counts).size !== 1) {
  console.error(`atom counts differ: ${langs.map((l, i) => `${l}=${counts[i]}`).join(' ')}`);
  process.exit(1);
}

// Ids already chosen by hand are worth more than anything derivable —
// `spec-pointer` says what the unit is, `v0-7-0-014` only says where it
// sits. Re-deriving a document's units keeps every id whose text still
// matches, and numbers the sentences carved out of it `-2`, `-3`, ….
// Text that repeats — `### Fixed` under every release — identifies no
// single unit, so those ids stay positional rather than all inheriting
// the first occurrence's name.
const existingIds = new Map();
if (existsSync(doc.source)) {
  const previous = (await import(pathToFileURL(doc.source).href)).default;
  const ambiguous = new Set();
  for (const unit of previous) {
    const text = Object.hasOwn(unit, 'common') ? unit.common : unit[langs[0]];
    if (existingIds.has(text)) ambiguous.add(text);
    else existingIds.set(text, unit.id);
  }
  const repeated = new Map();
  for (const atom of parsed[langs[0]]) {
    repeated.set(atom.text, (repeated.get(atom.text) ?? 0) + 1);
  }
  for (const [text, count] of repeated) if (count > 1) ambiguous.add(text);
  for (const text of ambiguous) existingIds.delete(text);
}

const usedIds = new Set();
const units = [];
let heading = 'preamble';
let ordinal = 0;
let kept = 0;
let split = 0;

for (let i = 0; i < counts[0]; i += 1) {
  const entry = Object.fromEntries(langs.map((lang) => [lang, parsed[lang][i]]));
  const atom = Object.fromEntries(langs.map((lang) => [lang, entry[lang].text]));

  const version = atom.en.match(/^## \[([^\]]+)\]/);
  if (version) {
    heading = `v${version[1].replace(/\./gu, '-')}`;
    ordinal = 0;
  }

  const pieces = Object.fromEntries(langs.map((lang) => [lang, sentences(atom[lang], lang)]));
  const sizes = langs.map((lang) => pieces[lang].length);

  // Sentence counts have to agree, and every boundary has to be spacing
  // the renderer can reproduce. When either fails, the whole atom stays
  // one unit — coarser than we want, but never wrong.
  const aligned = new Set(sizes).size === 1
    && langs.every((lang) => pieces[lang].every((p) => modeFor(p.glue) !== null));

  const emit = aligned ? sizes[0] : 1;
  if (aligned && sizes[0] > 1) split += 1;
  else kept += 1;

  if (!aligned && Math.max(...sizes) > 1 && process.argv.includes('--why')) {
    const bad = langs.filter((lang) => pieces[lang].some((p) => modeFor(p.glue) === null));
    console.log(`\n${heading} atom #${i}: sentences ${langs.map((l, k) => `${l}=${sizes[k]}`).join(' ')}`
      + (bad.length > 0 ? `  unreproducible spacing in ${bad.join(',')}` : ''));
    for (const lang of langs) {
      console.log(`  ${lang}: ${pieces[lang].map((p) => p.text.slice(0, 46).replace(/\n/gu, '\\n')).join('  ||  ')}`);
    }
  }

  const inherited = existingIds.get(atom[langs[0]]);

  for (let k = 0; k < emit; k += 1) {
    ordinal += 1;
    const text = Object.fromEntries(
      langs.map((lang) => [lang, aligned ? pieces[lang][k].text : atom[lang]]),
    );
    const join = Object.fromEntries(langs.map((lang) => [
      lang,
      k === 0 ? modeFor(entry[lang].glue) : modeFor(pieces[lang][k].glue),
    ]));
    const own = existingIds.get(text[langs[0]]);
    const wanted = own
      ?? (inherited ? `${inherited}-${k + 1}` : `${heading}-${String(ordinal).padStart(3, '0')}`);
    // Repeated text — a `---` rule, a shared heading — would otherwise
    // inherit one id twice, and duplicate ids are a hard failure.
    let id = wanted;
    for (let n = 2; usedIds.has(id); n += 1) id = `${wanted}-${n}`;
    usedIds.add(id);

    units.push({
      id,
      text,
      join: i === 0 && k === 0 ? null : join,
      common: langs.every((lang) => text[lang] === text[langs[0]]),
    });
  }
}

console.log(`atoms: ${counts[0]}  ->  units: ${units.length}`);
console.log(`  atoms split into sentences: ${split}`);
console.log(`  atoms kept whole:           ${kept}`);
console.log(`  language-independent units: ${units.filter((u) => u.common).length}`);

if (!process.argv.includes('--emit')) process.exit(0);

// ---------------------------------------------------------------------------
// Emit
// ---------------------------------------------------------------------------

const lit = (s) => '`' + s.replace(/\\/gu, '\\\\').replace(/`/gu, '\\`').replace(/\$\{/gu, '\\${') + '`';

const body = units.map((unit) => {
  const lines = [`  {`, `    id: '${unit.id}',`];
  if (unit.join) {
    const modes = langs.map((lang) => unit.join[lang]);
    lines.push(new Set(modes).size === 1
      ? `    join: '${modes[0]}',`
      : `    join: { ${langs.map((lang) => `${lang}: '${unit.join[lang]}'`).join(', ')} },`);
  }
  if (unit.common) lines.push(`    common: ${lit(unit.text[langs[0]])},`);
  else for (const lang of langs) lines.push(`    ${lang}: ${lit(unit.text[lang])},`);
  lines.push('  },');
  return lines.join('\n');
}).join('\n');

const header = `// Source of truth for ${langs.map((lang) => LANGS[lang]).join(', ')}.
//
// GENERATED FROM THIS FILE — never edit those artifacts by hand. Change
// the units here, then run
//   node tools/docs-gen/src/cli.mjs
// and verify with \`--check\`, which is what CI runs.
//
// One unit is one granular meaning: a heading, a sentence, a bullet, a
// table row. Its \`join\` records how it attaches to the unit before it —
// a blank line by default, a newline for \`tight\`, a space for \`flow\`,
// nothing for \`none\`. The map form exists because the three languages
// wrap their lines in different places. A unit is either a full
// translation set or a single \`common\` block for text that is identical
// in every language (code, tables, version headings).

export default [
`;

writeFileSync(doc.source, `${header}${body}\n];\n`, 'utf8');
console.log(`wrote ${doc.source}`);
