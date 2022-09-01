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
    pub portable: bool,
}

impl Parse for MakeFlatInfo {
    fn parse(input: ParseStream) -> Result<Self, ParseError> {
        let items = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        let mut info = MakeFlatInfo {
            sized: true,
            enum_type: None,
            portable: false,
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

        if let Some(lit) = params.remove("sized") {
            if let Lit::Bool(lit_bool) = lit {
                info.sized = lit_bool.value;
            } else {
                return Err(ParseError::new(
                    Span::call_site(),
                    "`sized` keyword requires bool value",
                ));
            }
        }
        if let Some(lit) = params.remove("enum_type") {
            if let Lit::Str(lit_str) = lit {
                info.enum_type = Some(Ident::new(&lit_str.value(), lit_str.span()));
            } else {
                return Err(ParseError::new(
                    Span::call_site(),
                    "`enum_type` keyword requires str value",
                ));
            }
        }
        if let Some(lit) = params.remove("portable") {
            if let Lit::Bool(lit_bool) = lit {
                info.portable = lit_bool.value;
            } else {
                return Err(ParseError::new(
                    Span::call_site(),
                    "`portable` keyword requires bool value",
                ));
            }
        }

        if params.len() != 0 {
            return Err(ParseError::new(
                Span::call_site(),
                format!(
                    "Unknown macro arguments: {:?}",
                    params.keys().collect::<Vec<_>>()
                ),
            ));
        }

        Ok(info)
    }
}
