mod util;

use chrono::NaiveDateTime;
use coins_database::PumpBondingCurve;
use coins_database::queries::pump_bonding_curve;
use util::setup_memory_pool;

#[tokio::test]
async fn bulk_create_and_list() {
    let pool = setup_memory_pool().await;
    let curves = vec![
        PumpBondingCurve {
            bonding_curve: "bc1".into(),
            mint: "mint_a".into(),
            first_seen: NaiveDateTime::default(),
        },
        PumpBondingCurve {
            bonding_curve: "bc2".into(),
            mint: "mint_b".into(),
            first_seen: NaiveDateTime::default(),
        },
    ];
    pump_bonding_curve::bulk_create(&pool, &curves)
        .await
        .unwrap();

    let list = pump_bonding_curve::list_bonding_curves(&pool)
        .await
        .unwrap();
    assert_eq!(list.len(), 2);
    assert!(list.contains(&"bc1".to_string()));
}

#[tokio::test]
async fn bulk_create_ignores_duplicates() {
    let pool = setup_memory_pool().await;
    let c = PumpBondingCurve {
        bonding_curve: "dup".into(),
        mint: "m".into(),
        first_seen: NaiveDateTime::default(),
    };
    pump_bonding_curve::bulk_create(&pool, &[c.clone()])
        .await
        .unwrap();
    pump_bonding_curve::bulk_create(&pool, &[c]).await.unwrap();
    assert_eq!(
        pump_bonding_curve::list_bonding_curves(&pool)
            .await
            .unwrap()
            .len(),
        1
    );
}
