#!/usr/bin/env bash
# Архитектурная проверка манифестов — без сети и без сборки.
#
# Зачем отдельно от cargo-тестов: если кто-то добавит `tauri` в `vela-core`,
# `cargo test` упадёт на разрешении зависимостей ДО запуска теста, и тест
# формально «не упал». Этот скрипт читает только текст манифестов, поэтому
# срабатывает всегда и в любом окружении.
#
# Правила (совпадают с tests/arch_rules.rs и tests/graph_rules.rs):
#   vela-core      — не зависит ни от UI/платформы, ни от других крейтов проекта
#   vela-ipc       — без tauri/windows/winit/wgpu
#   vela-platform  — без tauri
#
# Код возврата: 0 — нарушений нет, 1 — есть.
set -uo pipefail

cd "$(dirname "$0")/.." || exit 2

status=0

# Грепаем только секции зависимостей: простая эвристика по отступу не нужна,
# потому что имена крейтов не встречаются в полях [package].
check() {
  local crate_path="$1" crate_name="$2" pattern="$3" why="$4"
  local manifest="$crate_path/Cargo.toml"

  if [[ ! -f "$manifest" ]]; then
    echo "arch-check: $crate_name: манифест не найден ($manifest)" >&2
    status=1
    return
  fi

  local hits
  hits=$(grep -nE "^[[:space:]]*(${pattern})[[:space:]]*[.=]" "$manifest" || true)
  if [[ -n "$hits" ]]; then
    echo "arch-check: НАРУШЕНИЕ в $crate_name — $why" >&2
    echo "$hits" | sed 's/^/    /' >&2
    status=1
  fi
}

check crates/vela-core     vela-core     'tauri|tauri-build|windows|winit|wgpu|webkit2gtk|gtk' \
      "домен не должен зависеть от UI/платформы"
check crates/vela-core     vela-core     'vela-ipc|vela-platform|vela-cron' \
      "домен не должен зависеть от других крейтов проекта"
check crates/vela-ipc      vela-ipc      'tauri|windows|winit|wgpu' \
      "контракты обязаны собираться без графической среды"
check crates/vela-platform vela-platform 'tauri' \
      "платформенный слой не должен зависеть от фреймворка окон"

if [[ $status -eq 0 ]]; then
  echo "arch-check: ok"
fi
exit $status
