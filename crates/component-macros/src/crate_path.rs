use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

/// Resolve the GPUI API exposed to the crate where a macro is expanded.
///
/// `gpui-kit` is preferred because it re-exports GPUI and is the only direct
/// dependency required by kit consumers. The `gpui-pre` package fallback
/// preserves standalone `gpui-component` consumers, including dependencies
/// that rename that package to `gpui` (the conventional name).
pub(crate) fn gpui() -> syn::Result<TokenStream> {
    // [PiCell 포크 패치] 세 번째 후보 `gpui` 를 더한다. 업스트림은 crates.io 의
    // `gpui-pre*` 재배포본만 상정하지만, 우리는 로컬 `../zed` 체크아웃을 쓰고 그 패키지
    // 이름은 그냥 `gpui` 라서 위 두 이름으로는 절대 찾히지 않는다.
    for candidate in ["gpui-kit", "gpui-pre", "gpui"] {
        if let Ok(found) = crate_name(candidate) {
            return Ok(found_crate_path(found));
        }
    }
    Err(syn::Error::new(
        Span::call_site(),
        "IntoPlot requires a direct dependency on `gpui-kit`, `gpui-pre` or `gpui`",
    ))
}

fn found_crate_path(found: FoundCrate) -> TokenStream {
    match found {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
    }
}
