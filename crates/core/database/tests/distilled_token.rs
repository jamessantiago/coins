mod util;

use chrono::NaiveDateTime;
use coins_database::queries::distilled_token;
use coins_database::DistilledToken;
use util::setup_memory_pool;

fn sample_token(address: &str) -> DistilledToken {
    DistilledToken {
        address: address.into(),
        symbol: "".into(),
        name: "".into(),
        first_seen: NaiveDateTime::default(),
        last_seen: NaiveDateTime::default(),
        sources: "".into(),
        safety_score: None,
        liquidity_usd: None,
        volume_24h: None,
        fdv: None,
        narrative_clusters: "".into(),
        telegram_mentions: 0,
        cex_listed: false,
        research_conviction: None,
        dexscreener_url: "".into(),
        ranking_score: 0.0,
        price_change_24h: None,
        price_change_1h: None,
        vol_liq_ratio: None,
        buy_sell_ratio: None,
        updated_at: NaiveDateTime::default(),
    }
}

#[tokio::test]
async fn upsert_creates_and_updates() {
    let pool = setup_memory_pool().await;
    let mut t = sample_token("addr1");
    t.ranking_score = 10.0;
    distilled_token::upsert(&pool, &t).await.unwrap();

    let fetched = distilled_token::get_by_address(&pool, "addr1").await.unwrap().unwrap();
    assert_eq!(fetched.ranking_score, 10.0);

    t.ranking_score = 20.0;
    distilled_token::upsert(&pool, &t).await.unwrap();
    let fetched = distilled_token::get_by_address(&pool, "addr1").await.unwrap().unwrap();
    assert_eq!(fetched.ranking_score, 20.0);
}

#[tokio::test]
async fn get_by_address_returns_none_for_missing() {
    let pool = setup_memory_pool().await;
    let result = distilled_token::get_by_address(&pool, "missing").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn list_unscored_returns_only_zero_ranked() {
    let pool = setup_memory_pool().await;
    let mut t1 = sample_token("scored");
    t1.ranking_score = 5.0;
    let t2 = sample_token("unscored");
    distilled_token::upsert(&pool, &t1).await.unwrap();
    distilled_token::upsert(&pool, &t2).await.unwrap();

    let unscored = distilled_token::list_unscored(&pool).await.unwrap();
    assert_eq!(unscored.len(), 1);
    assert_eq!(unscored[0].address, "unscored");
}

#[tokio::test]
async fn update_ranking_score_modifies() {
    let pool = setup_memory_pool().await;
    let t = sample_token("addr_upd");
    distilled_token::upsert(&pool, &t).await.unwrap();

    assert!(distilled_token::update_ranking_score(&pool, "addr_upd", 99.0).await.unwrap());
    let fetched = distilled_token::get_by_address(&pool, "addr_upd").await.unwrap().unwrap();
    assert_eq!(fetched.ranking_score, 99.0);
}

#[tokio::test]
async fn count_returns_total() {
    let pool = setup_memory_pool().await;
    distilled_token::upsert(&pool, &sample_token("a")).await.unwrap();
    distilled_token::upsert(&pool, &sample_token("b")).await.unwrap();
    assert_eq!(distilled_token::count(&pool).await.unwrap(), 2);
}

#[tokio::test]
async fn list_narrative_clusters_distinct() {
    let pool = setup_memory_pool().await;
    let mut t1 = sample_token("a");
    t1.narrative_clusters = "defi".into();
    let mut t2 = sample_token("b");
    t2.narrative_clusters = "meme".into();
    distilled_token::upsert(&pool, &t1).await.unwrap();
    distilled_token::upsert(&pool, &t2).await.unwrap();

    let clusters = distilled_token::list_narrative_clusters(&pool).await.unwrap();
    assert_eq!(clusters.len(), 2);
}
