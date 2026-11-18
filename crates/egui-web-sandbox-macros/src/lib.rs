use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};
use syn::{ItemFn, parse_macro_input};

/// Keep the AppCreator factory available on every target and export `start()` on WASM.
#[proc_macro_attribute]
pub fn wasm_bindgen(args: TokenStream, input: TokenStream) -> TokenStream {
    let factory = parse_macro_input!(input as ItemFn);
    let sig = &factory.sig;
    if !args.is_empty()
        || !sig.inputs.is_empty()
        || !sig.generics.params.is_empty()
        || sig.asyncness.is_some()
        || sig.unsafety.is_some()
    {
        return syn::Error::new_spanned(
            sig,
            "expected #[wasm_bindgen] on a safe, synchronous fn() -> eframe::AppCreator<'static>",
        )
        .into_compile_error()
        .into();
    }

    let sandbox = match crate_name("egui-web-sandbox") {
        Ok(FoundCrate::Itself) => format_ident!("egui_web_sandbox"),
        Ok(FoundCrate::Name(name)) => format_ident!("{name}"),
        Err(error) => {
            return syn::Error::new_spanned(sig, error)
                .into_compile_error()
                .into();
        }
    };
    let factory_name = &sig.ident;
    quote! {
        #factory

        #[cfg(target_arch = "wasm32")]
        #[doc(hidden)]
        #[::#sandbox::__private::wasm_bindgen::prelude::wasm_bindgen(
            js_name = start,
            wasm_bindgen = ::#sandbox::__private::wasm_bindgen,
            wasm_bindgen_futures = ::#sandbox::__private::wasm_bindgen_futures,
            js_sys = ::#sandbox::__private::js_sys
        )]
        pub async fn __egui_web_sandbox_start() -> ::core::result::Result<
            ::#sandbox::web::WebApp,
            ::#sandbox::__private::wasm_bindgen::JsValue,
        > {
            ::#sandbox::web::launch(#factory_name()).await
        }
    }
    .into()
}
