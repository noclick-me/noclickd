use regex::Regex;
use unidecode::unidecode;

pub fn urlize_str(text: &str) -> String {
    // TODO: lazy_static
    let re = Regex::new(r" +").unwrap();
    let text = re
        .replace_all(&unidecode(text), "_")
        .to_string()
        .replace("%", "pct");
    let re = Regex::new(r"[^-/_~\.0-9a-zA-Z]+").unwrap();
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
            "AEneid_etude_Bei_Jing_shanana_genmaiCha_caeiounNOOI",
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
            "-1-deg3-EUR1/4"
        );
    }
}
