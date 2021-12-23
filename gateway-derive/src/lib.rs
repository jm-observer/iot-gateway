use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(SuperActionImpl)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_hello_macro(&ast)
}

fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl SuperAction for #name {
            fn _get_packet(&self) -> &MqttPacket {
                return &self.packet;
            }
            fn _get_global(&self) -> Arc<Global> {
                return self.global.clone();
            }
        }
    };
    gen.into()
}
