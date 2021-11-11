use crate::AsOption;
use proc_macro2::Span;
use std::fmt;
use std::fmt::Write;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Ident, Lit, LitStr, Meta, NestedMeta, Path};

use crate::utils::LitExt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ValueKind {
    // #[<name>]
    Name,
    // #[<name> = <value>]
    Equals,
    // #[<name>([<value>, <value>, <value>, ...])]
    List,
    // #[<name>(<value>)]
    SingleList,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueKind::Name => f.pad("`#[<name>]`"),
            ValueKind::Equals => f.pad("`#[<name> = <value>]`"),
            ValueKind::List => f.pad("`#[<name>([<value>, <value>, <value>, ...])]`"),
            ValueKind::SingleList => f.pad("`#[<name>(<value>)]`"),
        }
    }
}

fn to_ident(p: Path) -> syn::Result<Ident> {
    if p.segments.is_empty() {
        return Err(Error::new(
            p.span(),
            "cannot convert an empty path to an identifier",
        ));
    }

    if p.segments.len() > 1 {
        return Err(Error::new(
            p.span(),
            "the path must not have more than one segment",
        ));
    }

    if !p.segments[0].arguments.is_empty() {
        return Err(Error::new(
            p.span(),
            "the singular path segment must not have any arguments",
        ));
    }

    Ok(p.segments[0].ident.clone())
}

#[derive(Debug)]
pub struct Values {
    pub name: Ident,
    pub literals: Vec<Lit>,
    pub kind: ValueKind,
    pub span: Span,
}

impl Values {
    #[inline]
    pub fn new(name: Ident, kind: ValueKind, literals: Vec<Lit>, span: Span) -> Self {
        Values {
            name,
            literals,
            kind,
            span,
        }
    }
}

pub fn parse_values(attr: &Attribute) -> syn::Result<Values> {
    let meta = attr.parse_meta()?;

    match meta {
        Meta::Path(path) => {
            let name = to_ident(path)?;

            Ok(Values::new(name, ValueKind::Name, Vec::new(), attr.span()))
        }
        Meta::List(meta) => {
            let name = to_ident(meta.path)?;
            let nested = meta.nested;

            if nested.is_empty() {
                return Err(Error::new(attr.span(), "list cannot be empty"));
            }

            let mut lits = Vec::with_capacity(nested.len());

            for meta in nested {
                match meta {
                    NestedMeta::Lit(l) => lits.push(l),
                    NestedMeta::Meta(m) => match m {
                        Meta::Path(p) => {
                            let i = to_ident(p)?;
                            lits.push(Lit::Str(LitStr::new(&i.to_string(), i.span())))
                        }
                        Meta::List(_) | Meta::NameValue(_) => {
                            return Err(Error::new(attr.span(), "cannot nest a list, only accept literals and identifiers at this level"))
                        }
                    }
                }
            }

            let kind = if lits.len() == 1 {
                ValueKind::SingleList
            } else {
                ValueKind::List
            };

            Ok(Values::new(name, kind, lits, attr.span()))
        }
        Meta::NameValue(meta) => {
            let name = to_ident(meta.path)?;
            let lit = meta.lit;

            Ok(Values::new(name, ValueKind::Equals, vec![lit], attr.span()))
        }
    }
}

#[derive(Debug, Clone)]
struct DisplaySlice<'a, T>(&'a [T]);

impl<'a, T: fmt::Display> fmt::Display for DisplaySlice<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter().enumerate();

        match iter.next() {
            None => f.write_str("nothing")?,
            Some((idx, elem)) => {
                write!(f, "{}: {}", idx, elem)?;

                for (idx, elem) in iter {
                    f.write_char('\n')?;
                    write!(f, "{}: {}", idx, elem)?;
                }
            }
        }

        Ok(())
    }
}

#[inline]
fn is_form_acceptable(expect: &[ValueKind], kind: ValueKind) -> bool {
    if expect.contains(&ValueKind::List) && kind == ValueKind::SingleList {
        true
    } else {
        expect.contains(&kind)
    }
}

#[inline]
fn validate(values: &Values, forms: &[ValueKind]) -> syn::Result<()> {
    if !is_form_acceptable(forms, values.kind) {
        return Err(Error::new(
            values.span,
            format_args!(
                "the attribute must be in one of these forms:\n{}",
                DisplaySlice(forms)
            ),
        ));
    }

    Ok(())
}

#[inline]
pub fn parse<T: AttributeOption>(values: Values) -> syn::Result<T> {
    T::parse(values)
}

pub trait AttributeOption: Sized {
    const VALUE_KINDS: &'static [ValueKind];

    fn apply(lit: Option<&Lit>) -> syn::Result<Self>;

    fn parse(values: Values) -> syn::Result<Self> {
        validate(&values, Self::VALUE_KINDS)?;

        Self::apply(values.literals.get(0))
    }
}

impl<T: AttributeOption> AttributeOption for Vec<T> {
    const VALUE_KINDS: &'static [ValueKind] = &[ValueKind::List];

    fn apply(_lit: Option<&Lit>) -> syn::Result<Self> {
        Ok(vec![])
    }

    fn parse(values: Values) -> syn::Result<Self> {
        validate(&values, Self::VALUE_KINDS)?;

        values
            .literals
            .iter()
            .map(|lit| T::apply(Some(lit)))
            .collect()
    }
}

impl AttributeOption for String {
    const VALUE_KINDS: &'static [ValueKind] = &[ValueKind::Equals, ValueKind::SingleList];

    #[inline]
    fn apply(lit: Option<&Lit>) -> syn::Result<Self> {
        Ok(lit.unwrap().to_str())
    }
}

impl AttributeOption for bool {
    const VALUE_KINDS: &'static [ValueKind] = &[ValueKind::Name, ValueKind::SingleList];

    #[inline]
    fn apply(lit: Option<&Lit>) -> syn::Result<Self> {
        Ok(lit.map_or(true, |l| l.to_bool()))
    }
}

impl AttributeOption for Ident {
    const VALUE_KINDS: &'static [ValueKind] = &[ValueKind::SingleList];

    #[inline]
    fn apply(lit: Option<&Lit>) -> syn::Result<Self> {
        Ok(lit.unwrap().to_ident())
    }
}

impl AttributeOption for Option<String> {
    const VALUE_KINDS: &'static [ValueKind] =
        &[ValueKind::Name, ValueKind::Equals, ValueKind::SingleList];

    #[inline]
    fn apply(lit: Option<&Lit>) -> syn::Result<Self> {
        Ok(lit.map(|l| l.to_str()))
    }
}

impl<T: AttributeOption> AttributeOption for AsOption<T> {
    const VALUE_KINDS: &'static [ValueKind] = &[ValueKind::SingleList];

    #[inline]
    fn apply(_lit: Option<&Lit>) -> syn::Result<Self> {
        Ok(AsOption(None))
    }

    #[inline]
    fn parse(values: Values) -> syn::Result<Self> {
        Ok(AsOption(Some(T::parse(values)?)))
    }
}

macro_rules! attr_option_num {
    ($($n:ty),*) => {
        $(
        impl AttributeOption for $n {
            const VALUE_KINDS: &'static [ValueKind] = &[ValueKind::SingleList];

            fn apply(lit: Option<&Lit>) -> syn::Result<Self> {
                match lit.unwrap() {
                    Lit::Int(l) => l.base10_parse::<$n>(),
                    l => {
                        let s = l.to_str();
                        s.as_str().parse::<$n>().map_err(|_| Error::new(l.span(), "invalid integer"))
                    }
                }
            }
        }

        impl AttributeOption for Option<$n> {
            const VALUE_KINDS: &'static [ValueKind] = <$n as AttributeOption>::VALUE_KINDS;

            #[inline]
            fn apply(lit: Option<&Lit>) -> syn::Result<Self> {
                <$n as AttributeOption>::apply(lit).map(Some)
            }
        }
        )*
    };
}

attr_option_num!(u16, u32, u64, usize);
