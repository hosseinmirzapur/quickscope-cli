use crate::data::models::TokenDetail;

/// Known memecoin narratives and their trigger words.
const NARRATIVES: &[(&str, &[&str])] = &[
    ("AI Agent", &["ai", "agent", "artificial", "intelligence", "llm", "gpt", "neural"]),
    ("Dog", &["dog", "shib", "inu", "doge", "hound", "woof", "puppy"]),
    ("Cat", &["cat", "meow", "kitten", "feline", "purr", "mew"]),
    ("Frog", &["frog", "pepe", "peepo", "ribbit", "toad"]),
    ("Political", &["trump", "biden", "maga", "president", "vote", "dem", "gop", "election"]),
    ("Gaming", &["game", "play", "mmo", "rpg", "gamer", "esport"]),
    ("DeFi", &["defi", "finance", "swap", "yield", "farm", "stake", "lend", "borrow"]),
    ("Meme Classic", &["meme", "dank", "based", "chad", "wojak", "giga"]),
    ("Celebrity", &["elon", "musk", "celebrity", "famous", "kanye", "tate"]),
    ("SOL Ecosystem", &["sol", "solana", "spl", "lamport"]),
];

/// Detect which narrative(s) a token belongs to.
/// Returns the strongest match, or None.
pub fn detect_narrative(detail: &TokenDetail) -> Option<String> {
    let text = format!(
        "{} {} {}",
        detail.token.name.to_lowercase(),
        detail.token.symbol.to_lowercase(),
        detail.social_links.as_ref()
            .and_then(|s| s.description.as_deref())
            .unwrap_or("")
            .to_lowercase(),
    );

    let mut best: Option<(&str, usize)> = None;

    for (narrative, keywords) in NARRATIVES {
        let count = keywords.iter()
            .filter(|kw| text.contains(*kw))
            .count();
        if count > 0 && (best.is_none() || count > best.unwrap().1) {
            best = Some((narrative, count));
        }
    }

    best.map(|(name, _)| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_dog_narrative() {
        let detail = TokenDetail {
            token: crate::data::models::Token {
                name: "Doge Wif Hat".to_string(),
                symbol: "DWH".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let narrative = detect_narrative(&detail);
        assert!(narrative.is_some());
        assert_eq!(narrative.unwrap(), "Dog");
    }

    #[test]
    fn test_detect_ai_narrative() {
        let detail = TokenDetail {
            token: crate::data::models::Token {
                name: "Neural AI Agent".to_string(),
                symbol: "NAI".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let narrative = detect_narrative(&detail);
        assert_eq!(narrative.unwrap(), "AI Agent");
    }

    #[test]
    fn test_no_narrative() {
        let detail = TokenDetail {
            token: crate::data::models::Token {
                name: "Xyzzy".to_string(),
                symbol: "XYZ".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let narrative = detect_narrative(&detail);
        assert!(narrative.is_none());
    }

    #[test]
    fn test_frog_narrative() {
        let detail = TokenDetail {
            token: crate::data::models::Token {
                name: "Pepe the Frog".to_string(),
                symbol: "PEPE".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let narrative = detect_narrative(&detail);
        assert_eq!(narrative.unwrap(), "Frog");
    }
}