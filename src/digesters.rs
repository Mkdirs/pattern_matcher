use crate::Symbol;

/// Transforms a sequence of [Symbol] into a new type
pub trait Digester<S:Symbol>{
    type Output;
    
    /// Transforms [symbols](Symbol) into [Self::Output]
    fn digest(symbols: &[S]) -> Self::Output;
}

pub struct IntDigester;
pub struct StringDigester;


impl Digester<char> for IntDigester {
    type Output = isize;
    fn digest(symbols: &[char]) -> Self::Output {
        let symbols = symbols.iter().collect::<String>();
        isize::from_str_radix(&symbols, 10).expect(&format!("{symbols} is not base 10 !"))
    }
}

impl Digester<char> for StringDigester {
    type Output = String;
    fn digest(symbols: &[char]) -> Self::Output {
        symbols.iter().collect::<String>()
    }
}

