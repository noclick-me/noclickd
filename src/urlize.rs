use regex::Regex;

pub fn urlize_str(text: &str) -> String {
    // TODO: lazy_static
    let re = Regex::new(r"[^-/_~\.0-9a-zA-Z]").unwrap();
    let text = text.replace(" ", "_").replace("%", "pct");
    re.replace_all(&text, "-").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_chars() {
        let valid = "1234567890qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM-/_~.";
        assert_eq!(urlize_str(valid), valid);
    }

    #[test]
    fn unicode() {
        assert_eq!(
            urlize_str("Æneid étude 北亰 ᔕᓇᓇ げんまい茶 çáéíóúñÑÓÖÎ"),
            "-neid_-tude_--_---_-----_-----------",
            // TODO: use https://crates.io/crates/unidecode
            //"AEneid_etude_Bei_Jing_shanana_genmaiCha_caeiounNOOI"
        );
    }

    #[test]
    fn translated_symbols() {
        assert_eq!(urlize_str("%"), "pct");
    }

    #[test]
    fn untranslated_symbols() {
        assert_eq!(
            urlize_str("+={}[]()*&^`@#$\\|\"';:?!¹¿¡><,´¨>«»”“°³¤€¼"),
            "-----------------------------------------"
        );
    }
}
