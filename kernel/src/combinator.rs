use crate::parser::Parser;

/// パーサーを0回以上繰り返し適用し、結果を Vec で返す（Haskell の many）
/// 一度も成功しなくても、空の Vec で成功する
pub fn many0<T: 'static>(p: Parser<T>) -> Parser<Vec<T>> {
    Parser(Box::new(move |input: String| {
        let mut results = Vec::new();
        let mut rest = input;
        while let Some((value, next)) = p.run(rest.clone()) {
            // 入力を消費しないパーサー（pure など）で無限ループしないようにする
            if next.len() == rest.len() {
                break;
            }
            results.push(value);
            rest = next;
        }
        Some((results, rest))
    }))
}

/// パーサーを1回以上繰り返し適用し、結果を Vec で返す（Haskell の some）
/// 一度も成功しなければ失敗する
pub fn many1<T: 'static>(p: Parser<T>) -> Parser<Vec<T>> {
    Parser(Box::new(move |input: String| {
        // 最低1回は成功する必要がある
        let (first, mut rest) = p.run(input)?;
        let mut results = vec![first];
        while let Some((value, next)) = p.run(rest.clone()) {
            if next.len() == rest.len() {
                break;
            }
            results.push(value);
            rest = next;
        }
        Some((results, rest))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::satisfy::satisfy;

    fn digit() -> Parser<char> {
        satisfy(|c| c.is_ascii_digit())
    }

    #[test]
    fn many0_matches_zero_or_more() {
        let (parsed, rest) = many0(digit()).run("12a".to_string()).unwrap();
        assert_eq!(parsed, vec!['1', '2']);
        assert_eq!(rest, "a");

        // 0回でも成功する
        let (parsed, rest) = many0(digit()).run("abc".to_string()).unwrap();
        assert_eq!(parsed, vec![]);
        assert_eq!(rest, "abc");
    }

    #[test]
    fn many1_requires_at_least_one() {
        let (parsed, rest) = many1(digit()).run("12a".to_string()).unwrap();
        assert_eq!(parsed, vec!['1', '2']);
        assert_eq!(rest, "a");

        assert!(many1(digit()).run("abc".to_string()).is_none());
    }
}
