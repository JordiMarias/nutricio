pub fn normalize_for_search(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'á' | 'ä' | 'â' => 'a',
            'è' | 'é' | 'ë' | 'ê' => 'e',
            'ì' | 'í' | 'ï' | 'î' => 'i',
            'ò' | 'ó' | 'ö' | 'ô' => 'o',
            'ù' | 'ú' | 'ü' | 'û' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            '\'' | '’' | '`' | '´' | '-' | '_' | '/' | '.' | ',' | '(' | ')' | '%' | '"' | ':' | ';' | '!' | '?' => ' ',
            other => other,
        })
        .collect::<String>()
}

pub fn matches_search(target: &str, query: &str) -> bool {
    let q_clean = normalize_for_search(query);
    let q_tokens: Vec<&str> = q_clean.split_whitespace().collect();
    if q_tokens.is_empty() {
        return true;
    }
    let target_clean = normalize_for_search(target);
    q_tokens.iter().all(|&token| target_clean.contains(token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accent_and_token_search() {
        let name = "BONPREU Beguda d'arròs 0% sucres en cartró";
        assert!(matches_search(name, "arros"));
        assert!(matches_search(name, "arròs"));
        assert!(matches_search(name, "beguda arros"));
        assert!(matches_search(name, "beguda d'arros"));
        assert!(matches_search(name, "arros 0%"));
        assert!(matches_search(name, "cartro"));
        assert!(matches_search(name, "bonpreu"));
        assert!(matches_search(name, "BONPREU Beguda d'arròs 0% sucres en cartró"));
        assert!(matches_search(name, "bonpreu beguda d'arros 0% sucres en cartro"));
        assert!(!matches_search(name, "civada"));
    }
}
