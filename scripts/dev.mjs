#!/usr/bin/env node
// Cross-platform developer task runner for Gale.
//
// Usage:
//   node scripts/dev.mjs <task> [options]
//   pnpm run <task>            (aliases in package.json)
//
// Run `node scripts/dev.mjs help` for the list of tasks.

import { spawnSync } from 'node:child_process';
import { rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { delimiter, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import process from 'node:process';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const srcTauri = join(root, 'src-tauri');
const win = process.platform === 'win32';
const mac = process.platform === 'darwin';

// The toolchain CI lints with; keep in sync with .github/workflows/check.yaml.
// Newer clippy versions flag upstream code that CI does not.
const CI_TOOLCHAIN = '1.88.0';

// Lints that upstream code does not pass yet; keep in sync with .github/workflows/check.yaml
const CLIPPY_ARGS = [
	'--all-targets',
	'--',
	'-D',
	'warnings',
	'-A',
	'clippy::wrong-self-convention',
	'-A',
	'clippy::doc-lazy-continuation',
	'-A',
	'clippy::uninlined-format-args',
	'-A',
	'clippy::new-without-default'
];

function fail(message) {
	console.error(`\nerror: ${message}`);
	process.exit(1);
}

function have(cmd) {
	return spawnSync(cmd, ['--version'], { stdio: 'ignore', shell: win }).status === 0;
}

function run(cmd, args, opts = {}) {
	console.log(`\n> ${[cmd, ...args].join(' ')}`);
	const res = spawnSync(cmd, args, { stdio: 'inherit', cwd: root, shell: win, ...opts });
	if (res.status !== 0) process.exit(res.status ?? 1);
}

// cargo is not always on PATH (e.g. rustup installed without shell integration);
// fall back to the active toolchain reported by rustup.
function ensureCargo() {
	if (have('cargo')) return;

	const which = spawnSync('rustup', ['which', 'cargo'], { encoding: 'utf8', shell: win });
	if (which.status === 0) {
		process.env.PATH = dirname(which.stdout.trim()) + delimiter + process.env.PATH;
		if (have('cargo')) return;
	}

	fail('cargo not found - install Rust via https://rustup.rs');
}

function pnpm(args, opts = {}) {
	if (have('pnpm')) return run('pnpm', args, opts);
	if (have('npx')) return run('npx', ['pnpm', ...args], opts);
	fail('pnpm not found - install it from https://pnpm.io');
}

function cargo(args) {
	ensureCargo();
	run('cargo', args, { cwd: srcTauri });
}

// Runs clippy with the same toolchain as CI when it is installed, so results match.
// A separate target dir keeps the two toolchains from thrashing each other's artifacts.
function clippy() {
	const which = spawnSync('rustup', ['which', '--toolchain', CI_TOOLCHAIN, 'cargo'], {
		encoding: 'utf8',
		shell: win
	});

	if (which.status !== 0) {
		console.log(
			`note: CI lints with Rust ${CI_TOOLCHAIN} - run \`rustup toolchain install ${CI_TOOLCHAIN}\` for identical results`
		);
		cargo(['clippy', ...CLIPPY_ARGS]);
		return;
	}

	const bin = dirname(which.stdout.trim());
	run(join(bin, 'cargo'), ['clippy', ...CLIPPY_ARGS], {
		cwd: srcTauri,
		env: {
			...process.env,
			PATH: bin + delimiter + process.env.PATH,
			CARGO_TARGET_DIR: join(srcTauri, 'target', 'ci-lint')
		}
	});
}

function setup() {
	pnpm(['install', '--frozen-lockfile']);
	pnpm(['locale:compile']);
	ensureCargo();
	if (mac) run('rustup', ['target', 'add', 'x86_64-apple-darwin', 'aarch64-apple-darwin']);
	console.log('\nsetup done - run `node scripts/dev.mjs app` to start the app');
}

function lint() {
	pnpm(['lint']);
	clippy();
}

function verify() {
	pnpm(['locale:compile']);
	pnpm(['lint']);
	pnpm(['check']);
	clippy();
	cargo(['test']);
	console.log('\nverify passed - matches what CI checks on pull requests');
}

function build(args) {
	// local verification builds skip updater artifacts, which need the signing key
	const config = join(tmpdir(), 'gale-local-build.json');
	writeFileSync(config, JSON.stringify({ bundle: { createUpdaterArtifacts: false } }));

	const extra = [];
	let bundleDir = join(srcTauri, 'target', 'release', 'bundle');

	if (mac) {
		if (args.includes('--universal')) {
			run('rustup', ['target', 'add', 'x86_64-apple-darwin', 'aarch64-apple-darwin']);
			extra.push('--target', 'universal-apple-darwin');
			bundleDir = join(srcTauri, 'target', 'universal-apple-darwin', 'release', 'bundle');
		}
		extra.push('--bundles', 'app');
	} else if (win) {
		extra.push('--bundles', 'msi');
	}

	ensureCargo();
	pnpm(['tauri', 'build', '--config', config, ...extra]);
	console.log(`\nbundles are in ${bundleDir}`);
}

function clean(args) {
	for (const dir of ['build', '.svelte-kit']) {
		console.log(`removing ${dir}`);
		rmSync(join(root, dir), { recursive: true, force: true });
	}

	if (args.includes('--deep')) {
		console.log('removing src-tauri/target (--deep)');
		rmSync(join(srcTauri, 'target'), { recursive: true, force: true });
	}
}

const tasks = {
	setup: ['Install dependencies, compile locales and add Rust targets', setup],
	app: ['Run the full desktop app in dev mode', () => pnpm(['tauri', 'dev'])],
	web: ['Run the frontend only (vite dev server)', () => pnpm(['dev'])],
	check: ['Type-check the frontend (svelte-check)', () => pnpm(['check'])],
	test: ['Run the Rust test suite', () => cargo(['test'])],
	lint: ['Prettier check plus clippy with the CI flags', lint],
	fmt: ['Auto-format the frontend with prettier', () => pnpm(['format'])],
	verify: ['Run everything CI checks: lint, type-check, clippy and tests', verify],
	build: ['Release build for this platform (--universal for a mac universal binary)', build],
	clean: ['Delete frontend build artifacts (--deep also deletes the Rust target dir)', clean]
};

function help() {
	console.log('Gale developer tasks:\n');
	for (const [name, [desc]] of Object.entries(tasks)) {
		console.log(`  ${name.padEnd(8)} ${desc}`);
	}
	console.log('\nusage: node scripts/dev.mjs <task> [options]');
}

const [task, ...args] = process.argv.slice(2);

if (!task || task === 'help') {
	help();
} else if (tasks[task]) {
	tasks[task][1](args);
} else {
	console.error(`unknown task: ${task}\n`);
	help();
	process.exit(1);
}
