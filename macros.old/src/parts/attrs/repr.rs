use syn::{Data, DeriveInput, Ident, Meta, NestedMeta};

fn get_type(input: &DeriveInput) -> Option<Ident> {
    let panic_msg = "Bad `repr` attribute format";
    let mut enum_type = None;
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
                                enum_type.replace(ident.clone()).is_none(),
                                "`repr(..)` contains more than one enum type",
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
    enum_type
}

pub fn validate(input: &DeriveInput) {
    let repr = get_type(input);
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
    get_type(input).expect("Enum must have type set in `repr(C, ..)`")
}