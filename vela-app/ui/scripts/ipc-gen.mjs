#!/usr/bin/env node
/**
 * Генерация TypeScript-типов из контрактов Rust.
 *
 * Зачем скрипт, а не одна строка в package.json: нужно (1) выставить
 * TS_RS_EXPORT_DIR, (2) проверить, что генерация действительно изменила ровно
 * ожидаемые файлы, (3) не падать в CI на отсутствии cargo, а объяснить причину.
 *
 * # Почему генерация идёт во временный каталог
 *
 * Раньше скрипт запускал экспорт в настоящий каталог контрактов и сравнивал
 * снимок «до» и «после». Тесты `export_bindings.rs` пишут во временный каталог
 * (иначе проверка типов меняла бы то, что проверяет), поэтому «после» всегда
 * совпадало с «до»: команда сообщала «изменений нет», даже когда типы
 * разошлись. Теперь cargo пишет в `mkdtemp`, скрипт сравнивает результат с
 * рабочим деревом и только в режиме генерации переносит файлы.
 *
 * Использование:
 *   node scripts/ipc-gen.mjs           сгенерировать
 *   node scripts/ipc-gen.mjs --check   проверить, что генерация идемпотентна
 */
import { execFileSync } from 'node:child_process';
import {
  cpSync,
  existsSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  unlinkSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
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
 *
 * Список читается из crates/vela-ipc/contract.json — того же манифеста,
 * который используют Rust-тесты. Дублировать его здесь нельзя: расхождение
 * означало бы, что один инструмент считает файл чужим и удаляет его.
 */
const contractPath = join(repoRoot, 'crates', 'vela-ipc', 'contract.json');
const contract = JSON.parse(readFileSync(contractPath, 'utf8'));
if (!Array.isArray(contract.handwritten) || !Array.isArray(contract.generated)) {
  throw new Error('contract.json: ожидались массивы handwritten и generated');
}
if (resolve(repoRoot, contract.dir) !== outDir) {
  throw new Error(
    `contract.json: dir=${contract.dir} не совпадает с каталогом генерации ${outDir}`,
  );
}
const HANDWRITTEN = new Set(contract.handwritten);

/** Снимает слепок сгенерированных файлов: имя → содержимое. */
function snapshot(dir) {
  if (!existsSync(dir)) return new Map();
  const entries = readdirSync(dir).filter(
    (name) => name.endsWith('.ts') && !name.endsWith('.test.ts') && !HANDWRITTEN.has(name),
  );
  entries.sort();
  // `core.autocrlf=true` на Windows отдаёт рабочие файлы с CRLF, а ts-rs пишет
  // LF. Без нормализации любая генерация считала бы все файлы изменёнными.
  return new Map(
    entries.map((name) => [name, readFileSync(join(dir, name), 'utf8').replace(/\r\n/g, '\n')]),
  );
}

/**
 * Запускает экспорт контрактов в указанный каталог.
 *
 * `stdio: 'inherit'` оставляет прогресс cargo в выводе: при падении теста
 * пользователь видит причину, а не только код возврата.
 */
function runExport(dir) {
  execFileSync(
    'cargo',
    ['test', '-p', 'vela-ipc', '--features', 'ts-export', '--test', 'export_bindings'],
    {
      cwd: repoRoot,
      stdio: 'inherit',
      env: { ...process.env, TS_RS_EXPORT_DIR: dir },
    },
  );
}

const tempDir = mkdtempSync(join(tmpdir(), 'vela-ipc-'));
let status = 0;

try {
  runExport(tempDir);

  const expected = snapshot(tempDir);
  const actual = snapshot(outDir);

  const added = [...expected.keys()].filter((k) => !actual.has(k));
  const removed = [...actual.keys()].filter((k) => !expected.has(k));
  const changed = [...expected.keys()].filter(
    (k) => actual.has(k) && actual.get(k) !== expected.get(k),
  );

  const total = added.length + removed.length + changed.length;

  if (checkOnly) {
    if (total === 0) {
      console.log('ipc:gen --check: типы актуальны');
    } else {
      console.error('ipc:gen --check: контракты разошлись с Rust-типами.');
      for (const name of added) console.error(`  + ${name}`);
      for (const name of removed) console.error(`  - ${name}`);
      for (const name of changed) console.error(`  ~ ${name}`);
      console.error('Запустите `pnpm ipc:gen` и закоммитьте результат.');
      status = 1;
    }
  } else if (total === 0) {
    console.log('ipc:gen: изменений нет (идемпотентно)');
  } else {
    for (const name of [...added, ...changed]) {
      cpSync(join(tempDir, name), join(outDir, name));
    }
    // Файл, которого нет в контракте, — это стейл от переименованного типа.
    // Оставлять его нельзя: он продолжит импортироваться и выглядеть рабочим.
    for (const name of removed) {
      unlinkSync(join(outDir, name));
    }
    console.log(`ipc:gen: обновлено файлов ${total}`);
    for (const name of added) console.log(`  + ${name}`);
    for (const name of removed) console.log(`  - ${name}`);
    for (const name of changed) console.log(`  ~ ${name}`);
  }
} catch (error) {
  if (error.code === 'ENOENT') {
    console.error(
      'ipc:gen: не найден cargo. Установите Rust-тулчейн ' +
        '(https://rustup.rs) — контракты генерируются из Rust-типов.',
    );
    status = 2;
  } else {
    status = error.status ?? 1;
  }
} finally {
  rmSync(tempDir, { recursive: true, force: true });
}

process.exit(status);
