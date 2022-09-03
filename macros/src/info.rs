use std::collections::HashMap;
use syn::{
    parse::{Error as ParseError, Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Ident, Lit, Meta, Token,
};

#[derive(Clone, Debug)]
pub struct Info {
    pub sized: bool,
    pub enum_type: Option<Ident>,
    pub portable: bool,
    pub default: bool,
}

fn parse_lit_bool(lit: &Lit) -> Result<bool, ParseError> {
    if let Lit::Bool(lit_bool) = lit {
        Ok(lit_bool.value)
    } else {
        Err(ParseError::new(lit.span(), "keyword requires bool value"))
    }
}

fn parse_lit_ident(lit: &Lit) -> Result<Ident, ParseError> {
    if let Lit::Str(lit_str) = lit {
        Ok(Ident::new(&lit_str.value(), lit_str.span()))
    } else {
        Err(ParseError::new(
            lit.span(),
            "`enum_type` keyword requires str value",
        ))
    }
}

impl Parse for Info {
    fn parse(input: ParseStream) -> Result<Self, ParseError> {
        let items = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        let mut params = items
            .iter()
            .map(|meta| {
                if let Meta::NameValue(nv) = meta {
                    Ok((format!("{}", nv.path.get_ident().unwrap()), &nv.lit))
                } else {
                    Err(ParseError::new(
                        meta.span(),
                        "`name = value` format required",
                    ))
                }
            })
            .collect::<Result<HashMap<_, _>, ParseError>>()?;

        let info = Info {
            sized: if let Some(lit) = params.remove("sized") {
                parse_lit_bool(lit)?
            } else {
                true
            },
            enum_type: if let Some(lit) = params.remove("enum_type") {
                Some(parse_lit_ident(lit)?)
            } else {
                None
            },
            portable: if let Some(lit) = params.remove("portable") {
                parse_lit_bool(lit)?
            } else {
                false
            },
            default: if let Some(lit) = params.remove("default") {
                parse_lit_bool(lit)?
            } else {
                false
            },
        };

        if !params.is_empty() {
            return Err(ParseError::new(
                items.span(),
                format!(
                    "Unknown macro arguments: {:?}",
                    params.keys().collect::<Vec<_>>()
                ),
            ));
        }

        Ok(info)
    }
}
