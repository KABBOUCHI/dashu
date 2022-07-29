use proc_macro::TokenStream;

mod parse;

#[proc_macro]
pub fn ubig(input: TokenStream) -> TokenStream {
    parse::int::parse_integer::<false>(input.into()).into()
}

#[proc_macro]
pub fn ibig(input: TokenStream) -> TokenStream {
    parse::int::parse_integer::<true>(input.into()).into()
}

/// This macro only accepts binary or hexadecimal literals. It doesn't allow decimal literals because
/// the conversion is not always lossless. Therefore if you want to create an [FBig][dashu_float::FBig]
/// instance, use the [dbig] macro and then change the radix with [with_radix][dashu_float::FBig::with_radix]
/// 
/// The generated float has precision determined by length of input digits.
/// 
/// Note that literal `1.0e8` is both valid in decimal and hexadecimal representation, but it will
/// be parsed as a decimal float by default. If you want to parse it as a hexadecimal, you need to
/// specify the base argument.
#[proc_macro]
pub fn fbig(input: TokenStream) -> TokenStream {
    parse::float::parse_binary_float(input.into()).into()
}

#[proc_macro]
pub fn dbig(input: TokenStream) -> TokenStream {
    parse::float::parse_decimal_float(input.into()).into()
}
