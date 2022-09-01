use syn::Path;

pub fn eq(a: &Path, b: &Path) -> bool {
    a.leading_colon.is_some() == b.leading_colon.is_some()
        && a.segments.len() == b.segments.len()
        && a.segments
            .iter()
            .zip(b.segments.iter())
            .fold(true, |acc, (x, y)| {
                assert!(x.arguments.is_empty() && y.arguments.is_empty());
                acc && x.ident == y.ident
            })
}
