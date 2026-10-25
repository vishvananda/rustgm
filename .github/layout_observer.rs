fn main() {
    println!("{{\"Payload\":{},\"ValidatedRows\":{},\"Interface\":{},\"Columns\":{},\"SourceRows\":{},\"Attributes\":{},\"pointer_bytes\":{}}}",
        std::mem::size_of::<rustgm::support::crates::generics::Payload>(),
        std::mem::size_of::<rustgm::support::crates::generics::source::ValidatedRows>(),
        std::mem::size_of::<rustgm::support::crates::Interface>(),
        std::mem::size_of::<rustgm::support::crates::generics::source::Columns>(),
        std::mem::size_of::<rustgm::support::crates::generics::SourceRows>(),
        std::mem::size_of::<rustgm::support::crates::generics::source::HeaderAttributeColumns>(),
        std::mem::size_of::<usize>());
}
