/**
 * Общая настройка тестов: jsdom + матчеры.
 *
 * `@testing-library/jest-dom` намеренно не подключён: он тянет лишние 200 КБ
 * и меняет глобальные матчеры, а нам в P1 достаточно `expect` из vitest.
 * Когда появятся компонентные тесты (P4), матчеры добавятся точечно.
 */
import { afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';

afterEach(() => {
  cleanup();
});
