mod enum_impl;
mod packet_derive;
mod parsing;

#[proc_macro]
pub fn enum_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    enum_impl::enum_impl(input)
}

#[proc_macro_derive(Packet, attributes(packet))]
pub fn packet_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    packet_derive::packet_derive(input)
}