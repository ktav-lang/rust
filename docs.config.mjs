// Configuration for @ktav-lang/docs-gen (tools/docs-gen).
//
// The generator itself knows no languages, paths or document names —
// they all live here, which is what lets the same package serve other
// repositories in this ecosystem.
//
// Paths are relative to this file. Every artifact listed under `outputs`
// is GENERATED: edit the `source` module, then run
//   node tools/docs-gen/src/cli.mjs
// and verify with
//   node tools/docs-gen/src/cli.mjs --check
// which is what CI runs. Hand-editing an artifact is always wrong — the
// next build overwrites it and the check fails.

export default {
  languages: ['en', 'ru', 'zh'],

  documents: [
    {
      name: 'README',
      source: 'docs/content/readme.units.mjs',
      outputs: {
        en: 'README.md',
        ru: 'README.ru.md',
        zh: 'README.zh.md',
      },
    },
  ],
};
