use crate::parser::{ParseInput, Parser};

/// パーサーを0回以上繰り返し適用し、結果を Vec で返す（Haskell の many）
/// 一度も成功しなくても、空の Vec で成功する
pub fn many0<I, T>(p: Parser<I, T>) -> Parser<I, Vec<T>>
where
    I: ParseInput + 'static,
    T: 'static,
{
    Parser(Box::new(move |input: I| {
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
pub fn many1<I, T>(p: Parser<I, T>) -> Parser<I, Vec<T>>
where
    I: ParseInput + 'static,
    T: 'static,
{
    Parser(Box::new(move |input: I| {
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

/// パーサーの成否によらず成功し、結果を Option で返す（Haskell の optional）
pub fn optional<I, T>(p: Parser<I, T>) -> Parser<I, Option<T>>
where
    I: Clone + 'static,
    T: 'static,
{
    Parser(Box::new(move |input: I| match p.run(input.clone()) {
        Some((value, rest)) => Some((Some(value), rest)),
        None => Some((None, input)),
    }))
}

/// 区切りパーサーで区切られた1個以上の要素をパースする（Haskell の sepBy1）
/// 例: `sep_by1(expr, comma)` で `a, b, c`
pub fn sep_by1<I, T, S>(p: Parser<I, T>, separator: Parser<I, S>) -> Parser<I, Vec<T>>
where
    I: ParseInput + 'static,
    T: 'static,
    S: 'static,
{
    Parser(Box::new(move |input: I| {
        let (first, mut rest) = p.run(input)?;
        let mut results = vec![first];
        // 「区切り + 要素」の組で繰り返す。区切りの後に要素がなければそこで打ち切り、
        // 区切りは消費しない
        while let Some((_, after_sep)) = separator.run(rest.clone()) {
            match p.run(after_sep) {
                Some((value, next)) => {
                    results.push(value);
                    rest = next;
                }
                None => break,
            }
        }
        Some((results, rest))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::StrParser;
    use crate::satisfy::satisfy;

    fn digit() -> StrParser<char> {
        satisfy(|c| c.is_ascii_digit())
    }

    fn comma() -> StrParser<char> {
        satisfy(|c| c == ',')
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

    #[test]
    fn optional_always_succeeds() {
        let (parsed, rest) = optional(digit()).run("1a".to_string()).unwrap();
        assert_eq!(parsed, Some('1'));
        assert_eq!(rest, "a");

        let (parsed, rest) = optional(digit()).run("abc".to_string()).unwrap();
        assert_eq!(parsed, None);
        assert_eq!(rest, "abc");
    }

    #[test]
    fn sep_by1_parses_separated_items() {
        let (parsed, rest) = sep_by1(digit(), comma()).run("1,2,3a".to_string()).unwrap();
        assert_eq!(parsed, vec!['1', '2', '3']);
        assert_eq!(rest, "a");

        // 末尾の区切りは消費しない
        let (parsed, rest) = sep_by1(digit(), comma()).run("1,".to_string()).unwrap();
        assert_eq!(parsed, vec!['1']);
        assert_eq!(rest, ",");

        assert!(sep_by1(digit(), comma()).run("abc".to_string()).is_none());
    }
}
