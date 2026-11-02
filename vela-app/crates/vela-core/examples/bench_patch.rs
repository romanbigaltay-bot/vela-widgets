//! Замер AC-1: apply_patch на 50 элементах.
//!
//! Отдельный пример, а не тест: тест проверяет только потолок, а здесь нужна
//! цифра, пригодная для отчёта и для сравнения между коммитами.
//!
//! `expect` здесь законен и разрешён явно: это измерительный инструмент,
//! который запускают руками. Если он не может создать временный каталог или
//! открыть базу, падение с внятным сообщением — правильное поведение, а
//! `Result` из `main` дал бы бесполезный трейсбек без контекста замера.
#![allow(
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]

use std::time::Instant;
use vela_core::{Db, Element, ElementKind, ElementPatch, ElementsRepo, OpenPolicy, PatchOp, Rect};

fn main() {
    let dir = tempfile::tempdir().expect("каталог");
    let db = Db::open(dir.path().join("bench.db"), OpenPolicy::Recover).expect("открытие");
    vela_core::migrate(&db).expect("миграции");
    let repo = vela_core::SqliteElements::new(&db);

    let make = |i: usize| Element {
        id: format!("e{i}"),
        kind: ElementKind::Widget,
        widget_id: Some("core.clock".into()),
        monitor_id: "m1".into(),
        profile_id: None,
        frame: Rect::new(i as f64 * 10.0, 0.0, 240.0, 120.0),
        z: i as i32,
        rotation: 0.0,
        settings: serde_json::json!({}),
        state: serde_json::json!({}),
        created_at: 0,
        updated_at: 0,
    };

    let patches: Vec<ElementPatch> = (0..50)
        .map(|i| ElementPatch {
            id: format!("e{i}"),
            op: PatchOp::Upsert,
            element: Some(make(i)),
            frame: None,
            z: None,
            settings: None,
        })
        .collect();

    // Прогрев: первый вызов включает компиляцию подготовленных выражений.
    repo.apply_patch(patches.clone()).expect("прогрев");

    let mut best = u128::MAX;
    let mut total = 0u128;
    for _ in 0..20 {
        let started = Instant::now();
        repo.apply_patch(patches.clone()).expect("патч");
        let us = started.elapsed().as_micros();
        total += us;
        best = best.min(us);
    }

    println!("apply_patch, 50 элементов, 20 прогонов:");
    println!("  лучший: {:.2} мс", best as f64 / 1000.0);
    println!("  средний: {:.2} мс", (total / 20) as f64 / 1000.0);
    println!("  порог AC-1: 15 мс");
}
