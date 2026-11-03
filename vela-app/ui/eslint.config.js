import { readFileSync } from 'node:fs';
import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import globals from 'globals';

// Список сгенерированных контрактов берётся из манифеста, а не дублируется
// здесь: иначе новый тип начнёт линтоваться как рукописный код (или наоборот).
const contract = JSON.parse(
  readFileSync(new URL('../crates/vela-ipc/contract.json', import.meta.url), 'utf8'),
);
if (!Array.isArray(contract.generated) || !Array.isArray(contract.handwritten)) {
  throw new Error('contract.json: ожидались массивы generated и handwritten');
}
const generatedContracts = contract.generated.map((name) => `src/shared/api/${name}`);

/**
 * Линты фронтенда (ESLint 9, flat config).
 *
 * Три группы правил, по важности:
 * 1) архитектурные запреты проекта — ловят то, что тесты не увидят;
 * 2) строгость TypeScript;
 * 3) гигиена React.
 */
export default tseslint.config(
  {
    ignores: ['dist/**', 'node_modules/**', ...generatedContracts],
  },

  js.configs.recommended,
  ...tseslint.configs.recommended,

  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'module',
      globals: { ...globals.browser, ...globals.node },
    },
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      // --- Архитектурные запреты -------------------------------------------
      // Прямой вызов invoke запрещён: он обходит логирование, разворачивание
      // конверта и типизацию (ST-0102-03). Разрешён только в shared/api.
      'no-restricted-imports': [
        'error',
        {
          paths: [
            {
              name: '@tauri-apps/api/core',
              message:
                'Используйте call() из @/shared/api/client — прямой invoke обходит типы и логирование.',
            },
          ],
          patterns: [
            {
              group: ['**/features/*/internal/*'],
              message: 'Внутренности фичи нельзя импортировать снаружи — только через её index.ts.',
            },
          ],
        },
      ],

      // console.* запрещён везде, кроме логгера: иначе записи не попадут
      // в кольцевой буфер и в отчёт диагностики (T-0805).
      'no-console': 'error',

      // --- Строгость --------------------------------------------------------
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
      '@typescript-eslint/consistent-type-imports': [
        'error',
        { prefer: 'type-imports', fixStyle: 'inline-type-imports' },
      ],
      '@typescript-eslint/no-explicit-any': 'error',
      'no-restricted-syntax': [
        'error',
        {
          selector: 'TSNonNullExpression',
          message: 'Оператор ! скрывает реальную возможность null — обработайте случай явно.',
        },
        {
          selector: "CallExpression[callee.name='requestAnimationFrame']",
          message:
            'rAF только через useFrameLoop: иначе цикл не остановится и сгорит батарея (ST-0301-04).',
        },
        {
          selector: "CallExpression[callee.name='setInterval']",
          message:
            'Таймеры только через ctx.clock: иначе 9 виджетов держат 9 таймеров (ST-0401-02).',
        },
      ],
      eqeqeq: ['error', 'always'],
      'prefer-const': 'error',
      'no-var': 'error',

      // --- React ------------------------------------------------------------
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'warn',
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
    },
  },

  {
    // Сборочные скрипты исполняются в Node, а не в браузере: `process` и
    // `console` здесь законны. Отдельный блок, а не расширение списка files
    // у блока выше: `languageOptions.globals` — не аддитивное поле, и добавление
    // его к правилам UI-файлов затирало бы browser-глобалы (window, document).
    files: ['scripts/**/*.mjs', '*.config.{js,ts}', 'vitest.setup.ts'],
    languageOptions: {
      globals: { ...globals.node },
    },
    rules: {
      'no-console': 'off',
    },
  },

  {
    // Контракты IPC расширяются через слияние объявлений (`declare module`),
    // а для этого нужны интерфейсы без собственных членов. Правило про
    // пустые типы здесь мешает механизму, а не защищает от ошибки.
    files: ['src/shared/api/commands.ts', 'src/shared/api/events.ts'],
    rules: {
      '@typescript-eslint/no-empty-object-type': 'off',
    },
  },

  {
    // Единственные места, где разрешён прямой invoke и вывод в console.
    // Глобалы наследуются от блока для `**/*.{ts,tsx}` выше — здесь только правила.
    files: ['src/shared/lib/log.ts', 'src/shared/api/**/*.ts'],
    rules: {
      'no-console': 'off',
      'no-restricted-imports': 'off',
    },
  },

  {
    files: ['**/*.test.{ts,tsx}', 'vitest.setup.ts'],
    rules: {
      '@typescript-eslint/no-explicit-any': 'off',
      'no-restricted-syntax': 'off',
    },
  },
);
