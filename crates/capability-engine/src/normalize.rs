use unicode_normalization::UnicodeNormalization;

pub(crate) fn terms(input: &str) -> Vec<String> {
    let normalized = input.nfkc().flat_map(char::to_lowercase);
    let mut output = Vec::new();
    let mut word = String::new();
    let mut previous_cjk: Option<char> = None;

    for ch in normalized {
        if is_cjk(ch) {
            flush_word(&mut word, &mut output);
            output.push(ch.to_string());
            if let Some(previous) = previous_cjk {
                output.push(format!("{previous}{ch}"));
            }
            previous_cjk = Some(ch);
        } else if ch.is_alphanumeric() {
            previous_cjk = None;
            word.push(ch);
        } else {
            previous_cjk = None;
            flush_word(&mut word, &mut output);
        }
    }
    flush_word(&mut word, &mut output);
    output
}

fn flush_word(word: &mut String, output: &mut Vec<String>) {
    if !word.is_empty() {
        output.push(std::mem::take(word));
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch,
        '\u{3400}'..='\u{4dbf}'
            | '\u{4e00}'..='\u{9fff}'
            | '\u{f900}'..='\u{faff}'
            | '\u{20000}'..='\u{2fa1f}'
    )
}

#[cfg(test)]
mod tests {
    use super::terms;

    #[test]
    fn normalizes_ascii_and_cjk_bigrams() {
        let result = terms("Resume 会话恢复");
        assert!(result.contains(&"resume".to_owned()));
        assert!(result.contains(&"会话".to_owned()));
        assert!(result.contains(&"恢复".to_owned()));
    }
}
