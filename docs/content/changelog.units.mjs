// Source of truth for CHANGELOG.md, CHANGELOG.ru.md, CHANGELOG.zh.md.
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

import { preamble } from './changelog/preamble.mjs';
import { v0_8_0 } from './changelog/v0-8/v0-8-0.mjs';
import { v0_7_1 } from './changelog/v0-7/v0-7-1.mjs';
import { v0_7_0 } from './changelog/v0-7/v0-7-0.mjs';
import { v0_6_4 } from './changelog/v0-6/v0-6-4.mjs';
import { v0_6_3 } from './changelog/v0-6/v0-6-3.mjs';
import { v0_6_2 } from './changelog/v0-6/v0-6-2.mjs';
import { v0_6_1 } from './changelog/v0-6/v0-6-1.mjs';
import { v0_6_0 } from './changelog/v0-6/v0-6-0.mjs';
import { v0_5_0 } from './changelog/v0-5/v0-5-0.mjs';
import { v0_3_1 } from './changelog/v0-3/v0-3-1.mjs';
import { v0_3_0 } from './changelog/v0-3/v0-3-0.mjs';
import { v0_2_0 } from './changelog/v0-2/v0-2-0.mjs';
import { v0_1_5 } from './changelog/v0-1/v0-1-5.mjs';
import { v0_1_4 } from './changelog/v0-1/v0-1-4.mjs';
import { v0_1_3 } from './changelog/v0-1/v0-1-3.mjs';
import { v0_1_2 } from './changelog/v0-1/v0-1-2.mjs';
import { v0_1_1 } from './changelog/v0-1/v0-1-1.mjs';
import { v0_1_0 } from './changelog/v0-1/v0-1-0.mjs';

export default [
  ...preamble,
  ...v0_8_0,
  ...v0_7_1,
  ...v0_7_0,
  ...v0_6_4,
  ...v0_6_3,
  ...v0_6_2,
  ...v0_6_1,
  ...v0_6_0,
  ...v0_5_0,
  ...v0_3_1,
  ...v0_3_0,
  ...v0_2_0,
  ...v0_1_5,
  ...v0_1_4,
  ...v0_1_3,
  ...v0_1_2,
  ...v0_1_1,
  ...v0_1_0,
];
