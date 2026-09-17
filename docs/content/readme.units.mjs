// Source of truth for README.md, README.ru.md, README.zh.md.
//
// GENERATED FROM THIS FILE — never edit those artifacts by hand. Change
// the units here, then run
//   node tools/docs-gen/src/cli.mjs
// and verify with `--check`, which is what CI runs.
//
// One unit is one granular meaning: a heading, a sentence, a bullet, a
// table row. Its `join` records how it attaches to the unit before it —
// a blank line by default, a newline for `tight`, a space for `flow`,
// nothing for `none`. The map form exists because the three languages
// wrap their lines in different places. A unit is either a full
// translation set or a single `common` block for text that is identical
// in every language (code, tables, version headings).

import { intro } from './readme/language/intro.mjs';
import { name } from './readme/language/name.mjs';
import { motto } from './readme/language/motto.mjs';
import { rules } from './readme/language/rules.mjs';
import { values } from './readme/language/values.mjs';
import { compounds } from './readme/language/compounds.mjs';
import { corners } from './readme/language/corners.mjs';
import { rustUsage } from './readme/usage/rust-usage.mjs';
import { examples } from './readme/usage/examples.mjs';
import { roundTrip } from './readme/usage/round-trip.mjs';
import { formatting } from './readme/usage/formatting.mjs';
import { cabi } from './readme/project/cabi.mjs';
import { architecture } from './readme/project/architecture.mjs';
import { notDo } from './readme/project/not-do.mjs';
import { installation } from './readme/project/installation.mjs';
import { support } from './readme/project/support.mjs';
import { license } from './readme/project/license.mjs';
import { implementations } from './readme/project/implementations.mjs';

export default [
  ...intro,
  ...name,
  ...motto,
  ...rules,
  ...values,
  ...compounds,
  ...corners,
  ...rustUsage,
  ...examples,
  ...roundTrip,
  ...formatting,
  ...cabi,
  ...architecture,
  ...notDo,
  ...installation,
  ...support,
  ...license,
  ...implementations,
];
