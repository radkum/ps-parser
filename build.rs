use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::quote;

type DynError = Box<dyn std::error::Error>;
type DynResult<T> = core::result::Result<T, DynError>;

fn main() -> DynResult<()> {
    println!("cargo:rerun-if-changed=src/powershell.pest");
    // Generated here instead of `#[derive(Parser)]` so every rule can check the
    // per-thread parse budget; PEG backtracking is exponential on some inputs.
    let parser = pest_generator::derive_parser(
        quote! {
            #[grammar = "powershell.pest"]
            pub struct PowerShellSession;
        },
        false,
    );
    let out = std::path::Path::new(&std::env::var("OUT_DIR")?).join("powershell_parser.rs");
    std::fs::write(out, with_budget_checks(parser).to_string())?;
    Ok(())
}

/// Prepends a budget check to the body of every `fn ...(state ...)`.
fn with_budget_checks(tokens: TokenStream) -> TokenStream {
    let mut out = Vec::new();
    let mut in_rule_fn = false;
    let mut prev: Vec<TokenTree> = Vec::new();
    for tt in tokens {
        let tt = match tt {
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace && in_rule_fn => {
                in_rule_fn = false;
                let check = quote! { let state = crate::parser::parse_step(state)?; };
                let body = with_budget_checks(g.stream());
                TokenTree::Group(Group::new(Delimiter::Brace, quote! { #check #body }))
            }
            TokenTree::Group(g) => {
                let is_state_params = g.delimiter() == Delimiter::Parenthesis
                    && matches!(g.stream().into_iter().next(), Some(TokenTree::Ident(i)) if i == "state")
                    && matches!(prev.as_slice(), [.., TokenTree::Ident(f), TokenTree::Ident(_)] if f == "fn");
                in_rule_fn |= is_state_params;
                TokenTree::Group(Group::new(g.delimiter(), with_budget_checks(g.stream())))
            }
            tt => tt,
        };
        prev.push(tt.clone());
        out.push(tt);
    }
    out.into_iter().collect()
}
