use std::rc::Rc;

use kernel::parser::ParseInput;
use tokenizer::sql_token::Token;

/// 構文解析の入力となるトークン列
/// `Rc<Vec<Token>>` + 現在位置で表現することで、`clone()` を安価にしている
/// (コンビネータの `alt` などが入力を頻繁に複製するため)
#[derive(Debug, Clone, PartialEq)]
pub struct TokenStream {
    tokens: Rc<Vec<Token>>,
    position: usize,
}

impl TokenStream {
    /// トークン列から入力を作る。コメントは構文に関与しないためここで除外する
    pub fn new(tokens: Vec<Token>) -> Self {
        let tokens = tokens
            .into_iter()
            .filter(|t| !matches!(t, Token::Comment(_)))
            .collect();
        Self {
            tokens: Rc::new(tokens),
            position: 0,
        }
    }

    /// 現在位置のトークンを覗く(消費しない)
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// 1トークン進めた新しい入力を返す(元の入力は変更しない)
    pub fn advance(&self) -> Self {
        Self {
            tokens: Rc::clone(&self.tokens),
            position: (self.position + 1).min(self.tokens.len()),
        }
    }

    /// すべてのトークンを消費し終えたか
    pub fn is_empty(&self) -> bool {
        self.position >= self.tokens.len()
    }
}

impl ParseInput for TokenStream {
    fn len(&self) -> usize {
        self.tokens.len() - self.position
    }
}
