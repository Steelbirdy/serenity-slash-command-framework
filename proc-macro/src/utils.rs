use crate::SlashCommandFun;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::__private::TokenStream2;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{parenthesized, parse_quote, token::Mut, Error, Ident, Lifetime, Lit, Type};

pub struct Parenthesized<T>(pub Punctuated<T, Comma>);

impl<T: Parse> Parse for Parenthesized<T> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        Ok(Parenthesized(content.parse_terminated(T::parse)?))
    }
}

#[derive(Debug)]
pub struct AsOption<T>(pub Option<T>);

#[allow(dead_code)]
impl<T> AsOption<T> {
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> AsOption<U> {
        AsOption(self.0.map(f))
    }
}

impl<T> Default for AsOption<T> {
    #[inline]
    fn default() -> Self {
        AsOption(Default::default())
    }
}

#[derive(Debug)]
pub struct Argument {
    pub mutable: Option<Mut>,
    pub name: Ident,
    pub kind: Type,
}

impl ToTokens for Argument {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let Argument {
            mutable,
            name,
            kind,
        } = self;

        stream.extend(quote! {
            #mutable #name: #kind
        });
    }
}

#[inline]
pub fn generate_type_validation(have: Type, expect: Type) -> syn::Stmt {
    parse_quote! {
        serenity::static_assertions::assert_type_eq_all!(#have, #expect);
    }
}

pub fn create_declaration_validations(fun: &mut SlashCommandFun) -> syn::Result<()> {
    if fun.args.len() > 2 {
        return Err(Error::new(
            fun.args.last().unwrap().span(),
            format_args!("function's arity exceeds more than 3 arguments"),
        ));
    }

    let context: Type = parse_quote!(&serenity::client::Context);
    let interaction: Type = parse_quote!(
        &serenity::model::interactions::application_command::ApplicationCommandInteraction
    );

    let mut index = 0;

    let mut spoof_or_check = |kind: Type, name: &str| {
        match fun.args.get(index) {
            Some(x) => fun
                .body
                .insert(0, generate_type_validation(x.kind.clone(), kind)),
            None => fun.args.push(Argument {
                mutable: None,
                name: Ident::new(name, Span::call_site()),
                kind,
            }),
        }

        index += 1;
    };

    spoof_or_check(context, "_ctx");
    spoof_or_check(interaction, "_interaction");

    Ok(())
}

pub fn create_return_type_validation(r#fn: &mut SlashCommandFun, expect: Type) {
    let stmt = generate_type_validation(r#fn.ret.clone(), expect);
    r#fn.body.insert(0, stmt);
}

pub fn populate_fut_lifetimes_on_refs(args: &mut Vec<Argument>) {
    for arg in args {
        if let Type::Reference(r#ref) = &mut arg.kind {
            r#ref.lifetime = Some(Lifetime::new("'async_trait", Span::call_site()));
        }
    }
}

pub trait LitExt {
    fn to_str(&self) -> String;
    fn to_bool(&self) -> bool;
    fn to_ident(&self) -> Ident;
}

impl LitExt for Lit {
    fn to_str(&self) -> String {
        match self {
            Lit::Str(s) => s.value(),
            Lit::ByteStr(s) => String::from_utf8(s.value()).unwrap(),
            Lit::Char(c) => c.value().to_string(),
            Lit::Byte(b) => (b.value() as char).to_string(),
            _ => panic!("values must be a (byte)string or a char"),
        }
    }

    fn to_bool(&self) -> bool {
        if let Lit::Bool(b) = self {
            b.value
        } else {
            self.to_str()
                .parse()
                .unwrap_or_else(|_| panic!("expected bool from {:?}", self))
        }
    }

    #[inline]
    fn to_ident(&self) -> Ident {
        Ident::new(&self.to_str(), self.span())
    }
}

pub trait IdentExt: Sized {
    fn to_string_non_raw(&self) -> String;

    fn to_uppercase(&self) -> Self;

    fn with_suffix(&self, suf: &str) -> Ident;
}

impl IdentExt for Ident {
    #[inline]
    fn to_string_non_raw(&self) -> String {
        let ident_string = self.to_string();
        ident_string.trim_start_matches("r#").into()
    }

    #[inline]
    fn to_uppercase(&self) -> Self {
        format_ident!("{}", self.to_string_non_raw().to_uppercase())
    }

    #[inline]
    fn with_suffix(&self, suf: &str) -> Ident {
        format_ident!("{}_{}", self.to_uppercase(), suf)
    }
}

#[inline]
pub fn into_stream(e: Error) -> TokenStream {
    e.to_compile_error().into()
}

macro_rules! propagate_err {
    ($res:expr) => {{
        match $res {
            Ok(v) => v,
            Err(e) => return $crate::utils::into_stream(e),
        }
    }};
}

pub fn append_line(desc: &mut AsOption<String>, mut line: String) {
    if line.starts_with(' ') {
        line.remove(0);
    }

    let desc = desc.0.get_or_insert_with(String::default);

    match line.rfind("\\$") {
        Some(i) => {
            desc.push_str(line[..i].trim_end());
            desc.push(' ');
        }
        None => {
            desc.push_str(&line);
            desc.push('\n');
        }
    }
}
