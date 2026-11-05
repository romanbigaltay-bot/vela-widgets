/**
 * Автотесты токенов (TC-0103-01, AC-3, AC-4).
 *
 * Контраст считается по фактическим значениям из tokens.css, а не по
 * таблице в README: если кто-то поменяет цвет, тест упадёт.
 */

import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

/**
 * `import.meta.url` под Vitest — не file-URL (Vite подменяет его на http),
 * поэтому `fileURLToPath` падает. Ищем каталог от cwd: так тест работает и
 * из `vela-app/ui`, и из корня монорепозитория.
 */
function findStylesDir(): string {
  const candidates = [
    resolve(process.cwd(), 'src', 'styles'),
    resolve(process.cwd(), 'vela-app', 'ui', 'src', 'styles'),
  ];
  const found = candidates.find((dir) => existsSync(join(dir, 'tokens.css')));
  if (!found) throw new Error(`не найден tokens.css; cwd=${process.cwd()}`);
  return found;
}

const stylesDir = findStylesDir();
const tokensCss = readFileSync(join(stylesDir, 'tokens.css'), 'utf8');
const typographyCss = readFileSync(join(stylesDir, 'typography.css'), 'utf8');

interface Rgba {
  readonly r: number;
  readonly g: number;
  readonly b: number;
  readonly a: number;
}

const COMMON_TOKENS = [
  'font',
  'text-micro',
  'text-caption',
  'text-label',
  'text-body',
  'text-title',
  'text-display',
  'text-hero',
  'radius-chip',
  'radius-inner',
  'radius-panel',
  'radius-pill',
  'sp-1',
  'sp-2',
  'sp-3',
  'sp-4',
  'sp-5',
  'sp-6',
  'sp-7',
  'sp-8',
  'sp-9',
  'blur',
  'ease',
  'dur-in',
  'dur-out',
  'glass-noise',
] as const;

const THEME_TOKENS = [
  'bg',
  'bg-elev',
  'panel',
  'panel-hi',
  'panel-solid',
  'stroke',
  'stroke-hi',
  'text',
  'text-dim',
  'text-faint',
  'text-on-accent',
  'accent',
  'accent-text',
  'accent-soft',
  'accent-glow',
  'ok',
  'warn',
  'danger',
  'track',
  'inner',
  'shadow',
  'shadow-hi',
  'glow',
  'glass-tint',
  'glass-edge',
] as const;

function block(selector: string): string {
  const start = tokensCss.indexOf(selector);
  if (start < 0) throw new Error(`не найден блок ${selector}`);
  const open = tokensCss.indexOf('{', start);
  const close = tokensCss.indexOf('}', open);
  if (open < 0 || close < 0) throw new Error(`не разобран блок ${selector}`);
  return tokensCss.slice(open + 1, close);
}

function commonBlock(): string {
  const match = /:root\s*\{([\s\S]*?)\}/.exec(tokensCss);
  if (!match?.[1]) throw new Error('не найден общий блок :root');
  return match[1];
}

function token(blockText: string, name: string): string {
  const match = new RegExp(`--${name}:\\s*([^;]+);`).exec(blockText);
  if (!match?.[1]) throw new Error(`токен --${name} отсутствует`);
  return match[1].trim();
}

function parseColor(input: string): Rgba {
  const value = input.trim();
  const hex = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(value);
  if (hex?.[1]) {
    const raw = hex[1];
    const full =
      raw.length === 3
        ? raw
            .split('')
            .map((part) => part + part)
            .join('')
        : raw;
    return {
      r: parseInt(full.slice(0, 2), 16),
      g: parseInt(full.slice(2, 4), 16),
      b: parseInt(full.slice(4, 6), 16),
      a: 1,
    };
  }

  const rgb = /^rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)(?:\s*,\s*([\d.]+))?\s*\)$/i.exec(
    value,
  );
  if (rgb) {
    return {
      r: Number(rgb[1]),
      g: Number(rgb[2]),
      b: Number(rgb[3]),
      a: rgb[4] === undefined ? 1 : Number(rgb[4]),
    };
  }

  throw new Error(`неподдерживаемый цвет: ${value}`);
}

function channel(value: number): number {
  const normalized = value / 255;
  return normalized <= 0.03928 ? normalized / 12.92 : ((normalized + 0.055) / 1.055) ** 2.4;
}

function luminance(color: Rgba): number {
  return 0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b);
}

function contrast(first: Rgba, second: Rgba): number {
  const a = luminance(first);
  const b = luminance(second);
  const lighter = Math.max(a, b);
  const darker = Math.min(a, b);
  return (lighter + 0.05) / (darker + 0.05);
}

function ratio(blockText: string, foreground: string, background = 'bg'): number {
  return contrast(
    parseColor(token(blockText, foreground)),
    parseColor(token(blockText, background)),
  );
}

function cssFiles(dir: string): readonly string[] {
  const files: string[] = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      files.push(...cssFiles(full));
    } else if (entry.endsWith('.css')) {
      files.push(full);
    }
  }
  return files;
}

describe('токены', () => {
  it('содержит полный общий набор', () => {
    const root = commonBlock();
    for (const name of COMMON_TOKENS) {
      expect(token(root, name)).toBeTruthy();
    }
  });

  it('содержит полный набор в обеих темах', () => {
    for (const selector of [":root[data-theme='dark']", ":root[data-theme='light']"]) {
      const theme = block(selector);
      for (const name of THEME_TOKENS) {
        expect(token(theme, name), `${selector}: --${name}`).toBeTruthy();
      }
    }
  });

  it('держит контраст текстовых токенов', () => {
    for (const selector of [":root[data-theme='dark']", ":root[data-theme='light']"]) {
      const theme = block(selector);
      expect(ratio(theme, 'text'), `${selector}: text`).toBeGreaterThanOrEqual(4.5);
      expect(ratio(theme, 'text-dim'), `${selector}: text-dim`).toBeGreaterThanOrEqual(4.5);
      expect(ratio(theme, 'text-faint'), `${selector}: text-faint`).toBeGreaterThanOrEqual(3);
      expect(ratio(theme, 'accent'), `${selector}: accent`).toBeGreaterThanOrEqual(3);
      expect(ratio(theme, 'accent-text'), `${selector}: accent-text`).toBeGreaterThanOrEqual(4.5);
      expect(
        ratio(theme, 'text-on-accent', 'accent'),
        `${selector}: text-on-accent`,
      ).toBeGreaterThanOrEqual(4.5);
    }
  });

  it('обнуляет длительности при prefers-reduced-motion', () => {
    const media = /@media\s*\(prefers-reduced-motion:\s*reduce\)\s*\{([\s\S]*?)\}/.exec(tokensCss);
    expect(media?.[1]).toBeTruthy();
    const body = media?.[1] ?? '';
    expect(token(body, 'dur-in')).toBe('0ms');
    expect(token(body, 'dur-out')).toBe('0ms');
  });
});

describe('типографика', () => {
  function fontSize(className: string): number {
    const rule = new RegExp(`\\.${className}\\s*\\{([\\s\\S]*?)\\}`).exec(typographyCss);
    if (!rule?.[1]) throw new Error(`не найдено правило .${className}`);
    const reference = /font-size:\s*var\(--text-([a-z]+)\)/.exec(rule[1]);
    if (!reference?.[1]) throw new Error(`.${className}: font-size не через --text-*`);
    return Number.parseFloat(token(commonBlock(), `text-${reference[1]}`));
  }

  it('держит минимум 12 px и caption 14 px', () => {
    const order = [
      't-micro',
      't-caption',
      't-label',
      't-body',
      't-title',
      't-display',
      't-hero',
    ] as const;
    const sizes = order.map(fontSize);

    expect(sizes[0]).toBe(12);
    expect(sizes[1]).toBe(14);
    for (const size of sizes) {
      expect(size).toBeGreaterThanOrEqual(12);
    }
    for (let index = 1; index < sizes.length; index += 1) {
      expect(sizes[index]).toBeGreaterThan(sizes[index - 1] ?? 0);
    }
  });
});

describe('цвета вне tokens.css', () => {
  it('не содержит hex/rgb/hsl литералов', () => {
    const offenders: string[] = [];
    for (const file of cssFiles(stylesDir)) {
      if (file.endsWith('tokens.css')) continue;
      const text = readFileSync(file, 'utf8');
      const matches = text.match(/#[0-9a-f]{3,8}\b|rgba?\(|hsla?\(/gi);
      if (matches) offenders.push(`${file}: ${matches.join(', ')}`);
    }
    expect(offenders).toEqual([]);
  });
});
