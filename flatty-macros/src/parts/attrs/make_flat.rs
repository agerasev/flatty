use proc_macro2::Span;
use std::collections::HashMap;
use syn::{
    parse::{Error as ParseError, Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Lit, Meta, Token,
};

pub struct MakeFlatInfo {
    pub sized: bool,
    pub enum_type: Option<Ident>,
}

impl Parse for MakeFlatInfo {
    fn parse(input: ParseStream) -> Result<Self, ParseError> {
        let items = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        let mut info = MakeFlatInfo {
            sized: true,
            enum_type: None,
        };

        let mut params = items
            .iter()
            .map(|meta| {
                if let Meta::NameValue(nv) = meta {
                    Ok((format!("{}", nv.path.get_ident().unwrap()), &nv.lit))
                } else {
                    Err(ParseError::new(
                        Span::call_site(),
                        "`name = value` format required",
                    ))
                }
            })
            .collect::<Result<HashMap<_, _>, ParseError>>()?;

        info.sized = match params.remove("sized") {
            Some(lit) => {
                if let Lit::Bool(lit_bool) = lit {
                    Ok(lit_bool.value)
                } else {
                    Err(ParseError::new(
                        Span::call_site(),
                        "`sized` keyword requires bool value",
                    ))
                }
            }
            None => Ok(true),
        }?;
        info.enum_type = match params.remove("enum_type") {
            Some(lit) => {
                if let Lit::Str(lit_str) = lit {
                    Ok(Some(Ident::new(&lit_str.value(), lit_str.span())))
                } else {
                    Err(ParseError::new(
                        Span::call_site(),
                        "`enum_type` keyword requires str value",
                    ))
                }
            }
            None => Ok(None),
        }?;

        Ok(info)
    }
}
