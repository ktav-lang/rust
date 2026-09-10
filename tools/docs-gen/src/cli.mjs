#!/usr/bin/env node
// docs-gen CLI.
//
//   docs-gen [--config <path>]            regenerate every artifact
//   docs-gen --check [--config <path>]    verify byte-identical, no writes
//   docs-gen --help
//
// Default config path is docs.config.mjs in the current directory. The
// exit code is what CI reads: 0 when everything matches, 1 when an
// artifact is stale or a source is malformed, 2 on a usage error.

import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { DocsGenError, checkDocuments, loadConfig, writeDocuments } from './index.mjs';

const USAGE = `docs-gen — generate parallel multilingual markdown from source units

  docs-gen [--config <path>]          regenerate every configured artifact
  docs-gen --check [--config <path>]  verify byte-identical, write nothing
  docs-gen --help

Default config: ./docs.config.mjs
`;

function parseArgs(argv) {
  const opts = { check: false, config: 'docs.config.mjs', help: false };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--help' || arg === '-h') {
      opts.help = true;
    } else if (arg === '--check') {
      opts.check = true;
    } else if (arg === '--config') {
      const value = argv[i + 1];
      if (!value || value.startsWith('-')) {
        return { error: '--config needs a path' };
      }
      opts.config = value;
      i += 1;
    } else {
      return { error: `unknown argument ${arg}` };
    }
  }
  return opts;
}

export async function run(argv, io = process) {
  const opts = parseArgs(argv);

  if (opts.error) {
    io.stderr.write(`docs-gen: ${opts.error}\n\n${USAGE}`);
    return 2;
  }
  if (opts.help) {
    io.stdout.write(USAGE);
    return 0;
  }

  const config = await loadConfig(opts.config);

  if (!opts.check) {
    const { written } = await writeDocuments(config);
    io.stdout.write(`docs-gen: wrote ${written.length} artifact(s)\n`);
    return 0;
  }

  const { stale } = await checkDocuments(config);
  if (stale.length === 0) {
    io.stdout.write('docs-gen: all artifacts byte-identical to their sources\n');
    return 0;
  }

  for (const entry of stale) {
    // Point at the source, not the artifact: the artifact is output, and
    // "fix it" is exactly the wrong instinct to encourage.
    io.stderr.write(
      `docs-gen: ${entry.file} ${entry.reason}. This file is generated — `
      + `edit ${entry.source} and re-run docs-gen.\n`);
  }
  io.stderr.write(`docs-gen: ${stale.length} artifact(s) out of date\n`);
  return 1;
}

// fileURLToPath, not URL.pathname: on Windows the latter yields
// "/D:/..." and the comparison silently never matches.
const invokedDirectly = process.argv[1]
  && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);

if (invokedDirectly) {
  try {
    process.exitCode = await run(process.argv.slice(2));
  } catch (err) {
    if (err instanceof DocsGenError) {
      process.stderr.write(`docs-gen: ${err.message}\n`);
      process.exitCode = 1;
    } else {
      throw err;
    }
  }
}
