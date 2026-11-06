//! Замер AC-1 T-0104: первый старт с пустой БД и повторный прогон.
//!
//! Отдельный пример, как `bench_patch`: тест проверяет поведение, а здесь
//! нужна цифра для отчёта и для сравнения между коммитами. Каждый прогон
//! получает свежий временный каталог, поэтому `open` действительно создаёт
//! файл, а не переоткрывает существующий.
#![allow(clippy::expect_used)]

use std::time::{Duration, Instant};

const RUNS: usize = 20;

fn main() {
    let mut first = Vec::with_capacity(RUNS);
    let mut second = Vec::with_capacity(RUNS);

    for _ in 0..RUNS {
        let dir = tempfile::tempdir().expect("временный каталог");
        let path = dir.path().join("state.db");

        let started = Instant::now();
        let mut db = vela_core::Db::open(&path, vela_core::OpenPolicy::Recover).expect("открытие");
        let outcome = vela_core::migrate(&db).expect("миграция");
        first.push(started.elapsed());
        assert!(
            matches!(outcome, vela_core::MigrateOutcome::Applied { from: 0, .. }),
            "первый прогон должен применять миграции с нуля: {outcome:?}"
        );

        let started = Instant::now();
        let again = vela_core::migrate(&db).expect("повторная миграция");
        second.push(started.elapsed());
        assert!(
            matches!(again, vela_core::MigrateOutcome::UpToDate { .. }),
            "повторный прогон не должен трогать схему: {again:?}"
        );

        db.shutdown().expect("закрытие");
    }

    println!("T-0104 AC-1, {RUNS} прогонов на временной БД:");
    report("первый старт (open + migrate)", &first);
    report("повторный прогон (migrate)", &second);
    println!("порог AC-1: 50 мс на первый старт");
}

fn report(name: &str, samples: &[Duration]) {
    let best = samples.iter().min().copied().unwrap_or_default();
    let total: Duration = samples.iter().sum();
    let divisor = u32::try_from(samples.len()).unwrap_or(1);
    let average = total / divisor;
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let median = sorted.get(sorted.len() / 2).copied().unwrap_or_default();

    println!(
        "  {name}: лучший {:.2} мс, медиана {:.2} мс, средний {:.2} мс",
        ms(best),
        ms(median),
        ms(average)
    );
}

fn ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}
