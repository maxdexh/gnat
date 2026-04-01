use std::iter;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

trait ExtendExt<T>: Extend<T> + Sized {
    fn extended(mut self, iter: impl IntoIterator<Item = T>) -> Self {
        self.extend(iter);
        self
    }
}
impl<T, I: Extend<T>> ExtendExt<T> for I {}

macro_rules! spanned {
    ($ex:expr, $span:expr) => {{
        let mut tok = $ex;
        tok.set_span($span);
        ::proc_macro::TokenTree::from(tok)
    }};
}
fn pathseg(name: &str, span: Span) -> [TokenTree; 3] {
    [
        spanned!(Punct::new(':', Spacing::Joint), span),
        spanned!(Punct::new(':', Spacing::Alone), span),
        ident(name, span),
    ]
}
fn punct(ch: char, span: Span) -> TokenTree {
    spanned!(Punct::new(ch, Spacing::Alone), span)
}
fn group(stream: TokenStream, delim: Delimiter, delim_span: Span) -> TokenTree {
    spanned!(Group::new(delim, stream), delim_span)
}
fn litstr(str: &str, span: Span) -> TokenTree {
    spanned!(Literal::string(str), span)
}
fn ident(name: &str, span: Span) -> TokenTree {
    Ident::new(name, span).into()
}

struct SpanRange(Span, Span);
impl Default for SpanRange {
    fn default() -> Self {
        Self(Span::call_site(), Span::call_site())
    }
}
impl From<Span> for SpanRange {
    fn from(value: Span) -> Self {
        Self(value, value)
    }
}
impl<T: Into<SpanRange>> From<Option<T>> for SpanRange {
    fn from(value: Option<T>) -> Self {
        value.map(Into::into).unwrap_or_default()
    }
}
impl From<TokenStream> for SpanRange {
    fn from(value: TokenStream) -> Self {
        let mut iter = value.into_iter();
        iter.next().map_or_else(Default::default, |first| {
            let start = first.span();
            Self(start, iter.last().map_or(start, |last| last.span()))
        })
    }
}
impl From<&[TokenTree]> for SpanRange {
    fn from(value: &[TokenTree]) -> Self {
        match value {
            [] => Default::default(),
            [single] => single.span().into(),
            [start, .., end] => Self(start.span(), end.span()),
        }
    }
}
fn compile_error(msg: &str, span: impl Into<SpanRange>) -> TokenStream {
    let SpanRange(start, end) = span.into();
    pathseg("core", start)
        .into_iter()
        .chain(pathseg("compile_error", start))
        .chain([
            punct('!', start),
            spanned!(Group::new(Delimiter::Brace, litstr(msg, end).into()), end),
        ])
        .collect()
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn __apply(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut attr = Vec::from_iter(attr);

    let args = attr
        .iter()
        .position(|tok| matches!(&tok, TokenTree::Punct(p) if p.as_char() == '!'))
        .map(|i| attr.split_off(i + 1))
        .unwrap_or_else(|| {
            attr.push(punct('!', Span::call_site()));
            Default::default()
        });

    attr.extended([group(
        TokenStream::from(group(
            TokenStream::from_iter(args),
            Delimiter::Parenthesis,
            Span::call_site(),
        ))
        .extended(input),
        Delimiter::Brace,
        Span::call_site(),
    )])
    .into_iter()
    .collect()
}

fn single_tree(input: impl IntoIterator<Item = TokenTree>) -> Option<TokenTree> {
    let mut iter = input.into_iter();
    match (iter.next(), iter.next()) {
        (Some(single), None) => Some(single),
        _ => None,
    }
}

fn lit_impl(input: TokenStream) -> Result<TokenStream, TokenStream> {
    let mut input = input.into_iter();
    let mut last_arg_span = Span::call_site();
    let mut get_arg = || -> Result<_, _> {
        let arg = input
            .next()
            .ok_or_else(|| compile_error("Unexpected end of input", last_arg_span))?;

        let TokenTree::Group(mut arg) = arg else {
            return Err(compile_error("Expected group", arg.span()));
        };
        last_arg_span = arg.span_close();

        loop {
            match single_tree(arg.stream()) {
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::None => {
                    arg = g;
                }
                _ => return Ok(arg.stream()),
            }
        }
    };

    let lit = get_arg()?;
    let append = get_arg()?;
    let zero = get_arg()?;
    let one = get_arg()?;

    let Some(TokenTree::Literal(lit)) = single_tree(lit.clone()) else {
        return Err(compile_error("Expected literal", lit));
    };
    let span = lit.span();
    let lit = lit.to_string().replace("_", "");
    let lit = lit.as_str();

    let doit = |digits: &str, radix: u32| -> Result<TokenStream, Box<dyn std::error::Error>> {
        let mut bits = {
            let num = ibig::UBig::from_str_radix(digits, radix)?;
            (0..num.bit_len()).rev().map(move |i| num.bit(i))
        };

        let Some(true) = bits.next() else {
            debug_assert!(
                bits.all(|bit| !bit),
                "Logic Error: First bit was zero/nonexistent but number was non-zero"
            );
            return Ok(zero);
        };

        let append_depth = bits.len();

        // `Append<Append<...Append<Append<`
        let output = iter::repeat_n(
            append.extended(
                Some(punct('<', span)), //
            ),
            append_depth,
        );
        // `Append<Append<...Append<Append<1`
        let output = output.chain(
            // First bit, `1`
            Some(one.clone()),
        );
        // `Append<Append<...Append<Append<1, B1>, B2>...>, BN>`
        let output = output.chain({
            // `, B>`
            let punct_bits = [zero, one].map(|c| {
                Some(punct(',', span))
                    .into_iter()
                    .chain(c)
                    .chain(Some(punct('>', span)))
                    .collect::<TokenStream>()
            });
            // `, B1>, B2>, ...>, BN>`
            bits.map(move |bit| punct_bits[usize::from(bit)].clone())
        });

        Ok(output.collect())
    };

    match lit.split_at_checked(2) {
        Some(("0x", hex)) => doit(hex, 16),
        Some(("0o", oct)) => doit(oct, 8),
        Some(("0b", bin)) => doit(bin, 2),
        _ => doit(lit, 10),
    }
    .map_err(|err| compile_error(&err.to_string(), span))
}

#[doc(hidden)]
#[proc_macro]
pub fn __lit(input: TokenStream) -> TokenStream {
    match lit_impl(input) {
        Ok(out) => out,
        Err(out) => out,
    }
}

#[cfg(feature = "full")]
mod full;
macro_rules! export_full {
    (#[$attr:meta] $name:ident($($arg:ident),*)) => {
        #[doc(hidden)]
        #[$attr]
        #[cfg(feature = "full")]
        pub fn $name($($arg: TokenStream),*) -> TokenStream {
            full::$name($($arg.into()),*).unwrap_or_else(syn::Error::into_compile_error).into()
        }
    };
}
export_full! { #[proc_macro_attribute] nat_expr(attr, input) }
export_full! { #[proc_macro] expr(input) }
