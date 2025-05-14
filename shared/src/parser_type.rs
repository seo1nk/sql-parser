/// パース結果の型エイリアス
/// 成功した場合は、(パース済みの値, 残りの文字列) を返す
pub type ParseResult<T> = Option<(T, String)>;

/// 入力文字列を受け取って、ParseResultを返すパーサーの型エイリアス
/// 環境をキャプチャしない純粋な関数にのみ適用できる型
// type Parser<T> = fn(String) -> ParseResult<T>;

/// 入力文字列を受け取って、`ParseResult` を返すパーサーの型エイリアス  
/// `Box<dyn Fn(String) -> ParseResult<T>>` とすることで、環境をキャプチャするクロージャも扱えるようにする  
/// `type Parser<T> = Box<dyn Fn(String) -> ParseResult<T>>` を型でラップする
pub struct Parser<T>(pub Box<dyn Fn(String) -> ParseResult<T>>);

/// 中身の関数を呼び出すメソッドを実装する
impl<T> Parser<T> {
    /// `self.0` は Box<dyn Fn(String) -> ParseResult<T>> 型  
    /// `self.0` を呼び出すことで、Box の中の引数に String を取る関数を実行する
    pub fn run(&self, input: String) -> ParseResult<T> {
        (self.0)(input)
    }
}

/// パース結果の値部分に対して関数を適用する Functor
pub trait Functor {
    type Output;
    /// パース結果の値部分に対して関数を適用する  
    /// ParseResult の Some((a, remaining)) の a に対して関数 f :: a -> b を適用する
    fn map<F, B>(self, f: F) -> Parser<B>
    where
        // 出力の型をBに変換する関数を受け取る
        F: Fn(Self::Output) -> B + 'static,
        Self: Sized;
}

impl<T> Functor for Parser<T>
where
    T: 'static,
{
    type Output = T;
    fn map<F, B>(self, f: F) -> Parser<B>
    where
        F: Fn(T) -> B + 'static,
    {
        Parser(Box::new(move |input: String| {
            // 元のパーサーを実行して、結果に関数 T->B を適用
            self.run(input).map(|(a, rest)| (f(a), rest))
        }))
    }
}

/// Haskell の <*> の型  
/// Applicative を実装するには　Functor を実装している必要がある
pub trait Applicative: Functor {
    /// パーサーのContextに持ち上げる関数 a -> F a  
    /// pure(x: T) -> Parser<T>
    fn pure(x: Self::Output) -> Self;

    /// 関数を返すパーサーと値を返すパーサーを組み合わせる関数 f (a -> b) -> f a -> f b
    ///
    /// - `self`: 値 a を返すパーサー
    /// - `p` : 関数 f: Self::Output → B を返す
    ///
    /// `ap` を使うことで、関数を返すパーサーと値を返すパーサーを組み合わせて、最終的に b を返すパーサーを作る
    fn ap<B, F>(self, p: Parser<F>) -> Parser<B>
    where
        F: Fn(Self::Output) -> B + 'static,
        Self: Sized;
}

impl<T> Applicative for Parser<T>
where
    T: Clone + 'static,
{
    fn pure(x: Self::Output) -> Self {
        Parser(Box::new(move |input: String| Some((x.clone(), input))))
    }

    fn ap<B, F>(self, p: Parser<F>) -> Parser<B>
    where
        F: Fn(Self::Output) -> B + 'static,
    {
        let parse = move |input: String| match p.run(input) {
            // rest は p が入力から消費した後の残りの文字列に、関数 f: Fn(Self::Output) -> B を適用する
            Some((f, rest)) => self.run(rest).map(|(a, rest)| (f(a), rest)),
            None => None,
        };
        // 「関数 f を値 a に適用した結果 f(a)」を返すパーサー（Parser<B>）
        Parser(Box::new(parse))
    }
}
