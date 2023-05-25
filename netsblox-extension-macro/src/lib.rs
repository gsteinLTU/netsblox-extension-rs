use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn netsblox_extension_block(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn netsblox_extension_info(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn netsblox_extension_label_part(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn netsblox_extension_category(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}