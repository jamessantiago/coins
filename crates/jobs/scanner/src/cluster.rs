use std::collections::HashMap;

pub fn cluster_keywords() -> HashMap<&'static str, Vec<&'static str>> {
    let mut m = HashMap::new();
    m.insert(
        "AI",
        vec![
            "ai", "agent", "brain", "deep", "neural", "gpt", "llm", "intel", "mind", "tensor",
            "cortex", "genius", "smart", "learn", "autonomous", "llama", "grok", "compute",
        ],
    );
    m.insert(
        "MEME",
        vec![
            "meme", "doge", "pepe", "shib", "floki", "bonk", "wojak", "chad", "moon", "woof",
            "cat", "dog", "frog", "hat", "nut", "wif", "coin",
        ],
    );
    m.insert(
        "RWA",
        vec![
            "rwa", "real", "asset", "treasury", "tbill", "bond", "stable", "reit", "commodity",
        ],
    );
    m.insert(
        "DEFI",
        vec![
            "swap", "lend", "borrow", "yield", "farm", "stake", "pool", "liquid", "vault", "earn",
            "auto",
        ],
    );
    m.insert(
        "GAME",
        vec![
            "game", "gaming", "play", "metaverse", "guild", "raid", "quest", "rpg", "pixel",
            "arcade",
        ],
    );
    m.insert(
        "DEPIN",
        vec![
            "depin", "physical", "infra", "network", "iot", "sensor", "wifi", "map", "node",
            "edge", "deploy",
        ],
    );
    m.insert(
        "PRIVACY",
        vec!["privacy", "private", "zk", "zero", "anon", "secret", "shield", "mask"],
    );
    m.insert(
        "SOCIAL",
        vec![
            "social", "friend", "chat", "message", "post", "connect", "share", "fan", "community",
        ],
    );
    m.insert(
        "INFRA",
        vec![
            "layer2", "l2", "scroll", "arb", "base", "op", "zksync", "blast", "linea", "stack",
        ],
    );
    m
}

pub fn match_clusters(name: &str, symbol: &str) -> Vec<String> {
    let text = format!("{} {}", name, symbol).to_lowercase();
    let keywords = cluster_keywords();
    let mut matched = Vec::new();

    for (cluster_name, kws) in &keywords {
        for kw in kws {
            let pattern = format!(r"\b{}\b", regex::escape(kw));
            if let Ok(re) = regex::Regex::new(&pattern) {
                if re.is_match(&text) {
                    matched.push(cluster_name.to_string());
                    break;
                }
            }
        }
    }

    matched
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_ai() {
        let hits = match_clusters("Neural Agent Token", "BRAIN");
        assert!(hits.contains(&"AI".to_string()));
    }

    #[test]
    fn test_match_meme() {
        let hits = match_clusters("Doge Moon Cat", "PEPE");
        assert!(hits.contains(&"MEME".to_string()));
    }

    #[test]
    fn test_match_defi() {
        let hits = match_clusters("Yield Farm Token", "YLD");
        assert!(hits.contains(&"DEFI".to_string()));
    }

    #[test]
    fn test_no_match() {
        let hits = match_clusters("RandomThing", "XYZ");
        assert!(hits.is_empty());
    }

    #[test]
    fn test_multi_cluster() {
        let hits = match_clusters("AI Gaming Token", "AGENT");
        assert!(hits.contains(&"AI".to_string()));
        assert!(hits.contains(&"GAME".to_string()));
    }
}
