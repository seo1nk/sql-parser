/// パーサーの入力になれる型のトレイト
/// 文字列(字句解析)とトークン列(構文解析)の両方を同じコンビネータで扱うために、
/// 入力型を抽象化する
pub trait ParseInput: Clone {
    /// 残りの要素数。入力を消費しないパーサーの無限ループ検出に使う
    fn len(&self) -> usize;

    /// 入力を消費し終えたか
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl ParseInput for String {
    fn len(&self) -> usize {
        self.len()
    }
}

/// パース結果の型エイリアス
/// 成功した場合は、(パース済みの値, 残りの入力) を返す
pub type ParseResult<I, T> = Option<(T, I)>;

/// 入力 `I` を受け取って、`ParseResult` を返すパーサーの型エイリアス
/// `Box<dyn Fn(I) -> ParseResult<I, T>>` とすることで、環境をキャプチャするクロージャも扱えるようにする
/// 字句解析では `I = String`、構文解析では `I = TokenStream` を使う
pub struct Parser<I, T>(pub Box<dyn Fn(I) -> ParseResult<I, T>>);

/// 文字列を入力とするパーサーの型エイリアス(字句解析用)
pub type StrParser<T> = Parser<String, T>;

/// 中身の関数を呼び出すメソッドを実装する
impl<I, T> Parser<I, T> {
    /// `self.0` は Box<dyn Fn(I) -> ParseResult<I, T>> 型
    /// `self.0` を呼び出すことで、Box の中の関数を実行する
    pub fn run(&self, input: I) -> ParseResult<I, T> {
        (self.0)(input)
    }
}

/// パース結果の値部分に対して関数を適用する Functor
pub trait Functor {
    type Input;
    type Output;
    /// パース結果の値部分に対して関数を適用する
    /// ParseResult の Some((a, remaining)) の a に対して関数 f :: a -> b を適用する
    fn map<F, B>(self, f: F) -> Parser<Self::Input, B>
    where
        // 出力の型をBに変換する関数を受け取る
        F: Fn(Self::Output) -> B + 'static,
        Self: Sized;
}

/// Haskell の <*> の型
/// Applicative を実装するには　Functor を実装している必要がある
pub trait Applicative: Functor {
    /// パーサーのContextに持ち上げる関数 a -> F a
    /// pure(x: T) -> Parser<I, T>
    fn pure(x: Self::Output) -> Self;

    /// 関数を返すパーサーと値を返すパーサーを組み合わせる関数 f (a -> b) -> f a -> f b
    ///
    /// - `self`: 値 a を返すパーサー
    /// - `p` : 関数 f: Self::Output → B を返す
    ///
    /// `ap` を使うことで、関数を返すパーサーと値を返すパーサーを組み合わせて、最終的に b を返すパーサーを作る
    fn ap<B, F>(self, p: Parser<Self::Input, F>) -> Parser<Self::Input, B>
    where
        F: Fn(Self::Output) -> B + 'static,
        Self: Sized;
}

/// Haskell の <|>
pub trait Alternative: Applicative {
    /// 常に失敗するパーサー（Haskellの empty に相当）
    fn empty() -> Self;
    /// 左が失敗したら右を試す
    /// (<|>) :: f a -> f a -> f a
    fn alt(self, other: Self) -> Self;
}

/// Haskell の >>=
pub trait Monad: Applicative {
    /// 前のパーサーの結果を使って、次に実行するパーサーを決める
    /// (>>=) :: m a -> (a -> m b) -> m b
    fn and_then<B, F>(self, f: F) -> Parser<Self::Input, B>
    where
        F: Fn(Self::Output) -> Parser<Self::Input, B> + 'static,
        B: 'static,
        Self: Sized;
}

/// Haskellの `$>`
pub trait RightFunctor: Functor {
    /// 左の結果を無視して右の結果を返す
    fn replace_with<U>(self, value: U) -> Parser<Self::Input, U>
    where
        U: Clone + 'static,
        Self: Sized;
}

impl<I, T> Functor for Parser<I, T>
where
    I: 'static,
    T: 'static,
{
    type Input = I;
    type Output = T;
    fn map<F, B>(self, f: F) -> Parser<I, B>
    where
        F: Fn(T) -> B + 'static,
    {
        Parser(Box::new(move |input: I| {
            // 元のパーサーを実行して、結果に関数 T->B を適用
            self.run(input).map(|(a, rest)| (f(a), rest))
        }))
    }
}

impl<I, T> Applicative for Parser<I, T>
where
    I: Clone + 'static,
    T: Clone + 'static,
{
    fn pure(x: Self::Output) -> Self {
        Parser(Box::new(move |input: I| Some((x.clone(), input))))
    }

    fn ap<B, F>(self, p: Parser<I, F>) -> Parser<I, B>
    where
        F: Fn(Self::Output) -> B + 'static,
    {
        let parse = move |input: I| match p.run(input) {
            // rest は p が入力から消費した後の残りの入力に、関数 f: Fn(Self::Output) -> B を適用する
            Some((f, rest)) => self.run(rest).map(|(a, rest)| (f(a), rest)),
            None => None,
        };
        // 「関数 f を値 a に適用した結果 f(a)」を返すパーサー（Parser<I, B>）
        Parser(Box::new(parse))
    }
}

impl<I, T> Alternative for Parser<I, T>
where
    I: Clone + 'static,
    T: Clone + 'static,
{
    fn empty() -> Self {
        Parser(Box::new(|_| None))
    }

    fn alt(self, other: Self) -> Self {
        Parser(Box::new(move |input: I| match self.run(input.clone()) {
            Some(result) => Some(result),
            None => other.run(input),
        }))
    }
}

impl<I, T> Monad for Parser<I, T>
where
    I: Clone + 'static,
    T: Clone + 'static,
{
    fn and_then<B, F>(self, f: F) -> Parser<I, B>
    where
        F: Fn(T) -> Parser<I, B> + 'static,
        B: 'static,
    {
        Parser(Box::new(move |input: I| {
            // 先に self を実行し、その結果 a から f(a) で次のパーサーを作って残りに適用する
            let (a, rest) = self.run(input)?;
            f(a).run(rest)
        }))
    }
}

impl<I, T> RightFunctor for Parser<I, T>
where
    I: 'static,
    T: 'static,
{
    fn replace_with<U>(self, value: U) -> Parser<I, U>
    where
        U: Clone + 'static,
    {
        self.map(move |_| value.clone())
    }
}
