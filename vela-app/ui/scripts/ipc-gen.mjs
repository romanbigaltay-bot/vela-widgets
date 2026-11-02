#!/usr/bin/env node
/**
 * Генерация TypeScript-типов из контрактов Rust.
 *
 * Зачем скрипт, а не одна строка в package.json: нужно (1) выставить
 * TS_RS_EXPORT_DIR относительно корня репозитория, (2) проверить, что
 * генерация действительно изменила ровно ожидаемые файлы, (3) не падать
 * в CI на отсутствии cargo, а объяснить причину.
 *
 * Использование:
 *   node scripts/ipc-gen.mjs           сгенерировать
 *   node scripts/ipc-gen.mjs --check   проверить, что генерация идемпотентна
 */
import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const uiRoot = resolve(here, '..');
const repoRoot = resolve(uiRoot, '..');
const outDir = join(uiRoot, 'src', 'shared', 'api');

const checkOnly = process.argv.includes('--check');

/**
 * Файлы, которые живут в каталоге контрактов, но пишутся руками.
 * Они не участвуют в сравнении: иначе `--check` будет считать их «удалёнными»
 * при каждой генерации, потому что ts-rs их не создаёт.
 */
// Список обязан совпадать с HANDWRITTEN в
// crates/vela-ipc/tests/export_bindings.rs: расхождение означает, что один
// из двух инструментов считает файл чужим и удаляет его.
const HANDWRITTEN = new Set([
  'index.ts',
  'client.ts',
  'client.test.ts',
  'events.ts',
  'events.test.ts',
  'commands.ts',
  'transport.ts',
  'version.ts',
  'useEvent.ts',
  'useEvent.test.ts',
]);

/** Снимает слепок сгенерированных файлов: имя → содержимое. */
function snapshot(dir) {
  if (!existsSync(dir)) return new Map();
  const entries = readdirSync(dir).filter(
    (name) => name.endsWith('.ts') && !name.endsWith('.test.ts') && !HANDWRITTEN.has(name),
  );
  entries.sort();
  return new Map(entries.map((name) => [name, readFileSync(join(dir, name), 'utf8')]));
}

function runExport() {
  try {
    execFileSync(
      'cargo',
      ['test', '-p', 'vela-ipc', '--features', 'ts-export', '--test', 'export_bindings'],
      {
        cwd: repoRoot,
        stdio: 'inherit',
        env: { ...process.env, TS_RS_EXPORT_DIR: outDir },
      },
    );
  } catch (error) {
    if (error.code === 'ENOENT') {
      console.error(
        'ipc:gen: не найден cargo. Установите Rust-тулчейн ' +
          '(https://rustup.rs) — контракты генерируются из Rust-типов.',
      );
      process.exit(2);
    }
    process.exit(error.status ?? 1);
  }
}

const before = snapshot(outDir);
runExport();
const after = snapshot(outDir);

const added = [...after.keys()].filter((k) => !before.has(k));
const removed = [...before.keys()].filter((k) => !after.has(k));
const changed = [...after.keys()].filter((k) => before.has(k) && before.get(k) !== after.get(k));

const total = added.length + removed.length + changed.length;

if (checkOnly) {
  if (total === 0) {
    console.log('ipc:gen --check: типы актуальны');
    process.exit(0);
  }
  console.error('ipc:gen --check: контракты разошлись с Rust-типами.');
  for (const name of added) console.error(`  + ${name}`);
  for (const name of removed) console.error(`  - ${name}`);
  for (const name of changed) console.error(`  ~ ${name}`);
  console.error('Запустите `pnpm ipc:gen` и закоммитьте результат.');
  process.exit(1);
}

if (total === 0) {
  console.log('ipc:gen: изменений нет (идемпотентно)');
} else {
  console.log(`ipc:gen: обновлено файлов ${total}`);
  for (const name of added) console.log(`  + ${name}`);
  for (const name of removed) console.log(`  - ${name}`);
  for (const name of changed) console.log(`  ~ ${name}`);
}
