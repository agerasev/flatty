use proc_macro2::Span;
use std::collections::HashMap;
use syn::{
    parse::{Error as ParseError, Parse, ParseStream},
    punctuated::Punctuated,
    Data, DeriveInput, Ident, Lit, Meta, NestedMeta, Token,
};

fn get_repr(input: &DeriveInput) -> Option<Ident> {
    let panic_msg = "Bad `repr` attribute format";
    let mut output = None;
    let mut has_repr = false;
    for attr in &input.attrs {
        if attr.path.is_ident("repr") {
            has_repr = true;
            if let Meta::List(meta_list) = attr.parse_meta().expect(panic_msg) {
                let mut has_repr_c = false;
                for nm in meta_list.nested {
                    if let NestedMeta::Meta(m) = nm {
                        let ident = m.path().get_ident().expect(panic_msg);
                        if ident == "C" {
                            has_repr_c = true;
                        } else {
                            assert!(
                                output.replace(ident.clone()).is_none(),
                                "`repr(..)` contains more than one type",
                            );
                        }
                    } else {
                        panic!("{}", panic_msg);
                    }
                }
                assert!(has_repr_c, "User-defined types must be `repr(C)`");
            } else {
                panic!("{}", panic_msg);
            }
        }
    }
    assert!(
        has_repr,
        "User-defined types must have `repr(..)` attribute",
    );
    output
}

pub fn validate_repr(input: &DeriveInput) {
    let repr = get_repr(input);
    match input.data {
        Data::Struct(_) | Data::Union(_) => {
            assert!(repr.is_none(), "Struct must be only `repr(C)`");
        }
        Data::Enum(_) => {
            const ALLOWED_TYPES: [&str; 4] = ["u8", "u16", "u32", "u64"];
            let ty = repr.expect("Enum must be `repr(C, x)` where `x` is the type of enum");
            assert!(
                ALLOWED_TYPES.iter().any(|x| ty == x),
                "Enum type must be one of {:?} not \"{}\"",
                ALLOWED_TYPES,
                ty
            );
        }
    }
}

pub fn get_enum_type(input: &DeriveInput) -> Ident {
    get_repr(input).expect("Enum must have type set in `repr(C, ..)`")
}

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
