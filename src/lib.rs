use std::{error::Error, fmt::{Debug, Display}, num::NonZeroUsize, process::Termination};

#[cfg(test)]
mod tests;

/// A struct representing a group of symbols
#[derive(Debug, PartialEq, Clone)]
pub struct SymbolGroup<'a, Symbol>{
    pub accepted_symbols: &'a [Symbol],
    pub description: &'a str
}

impl<'a, Symbol> SymbolGroup<'a, Symbol> where Symbol: Debug+PartialEq{
        /// Tells if a symbol is a part of the group or not
        pub fn accept(&self, symbol: &Symbol) -> bool {
            self.accepted_symbols.contains(symbol)
        }
}

#[derive(Debug, Clone, PartialEq)]
/// This structure helps you building a pattern matching pipeline
pub struct MatchingPipeline<Symbol> where Symbol: PartialEq+Clone{
    matched: Vec<Symbol>,
    unmatched: Vec<Symbol>,
    reached_eos: bool
}

impl <Symbol> Termination for MatchingPipeline<Symbol> where Symbol: PartialEq+Clone{
    fn report(self) -> std::process::ExitCode {
        std::process::ExitCode::SUCCESS
    }
}

impl From<&str> for MatchingPipeline<char>{
    fn from(value: &str) -> Self {
        MatchingPipeline::new(value.chars())
    }
}

#[derive(Debug, PartialEq)]
pub enum PipelineError<'a, Symbol> where Symbol: Debug{
    UnexpectedEos,
    WrongSymbol{
        expected: &'a Symbol,
        actual: Symbol
    },
    WrongPattern{
        expected: &'a [Symbol],
        actual: Vec<Symbol>
    },

    SymbolNotPartOfGroup{
        expected: SymbolGroup<'a, Symbol>,
        actual: Symbol
    },

    SymbolNotMatchAnyOf{
        expected: &'a [Symbol],
        actual: Symbol
    }

}
impl<'a, Symbol> Display for PipelineError<'a, Symbol> where Symbol: Debug+PartialEq{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self{
            &Self::UnexpectedEos => write!(f, "Unexpected end of stream"),
            &Self::WrongSymbol { expected, actual } => write!(f, "Expected {expected:?} but instead got {actual:?}"),
            &Self::WrongPattern { expected, actual } => write!(f, "Expected pattern {expected:?} but instead got {actual:?}"),
            &Self::SymbolNotMatchAnyOf { expected, actual } => write!(f, "Expected one of {expected:?} but instead got {actual:?}"),
            &Self::SymbolNotPartOfGroup { expected, actual } => write!(f, "Expected one of {0:?} but instead got {actual:?}", expected.description)
        }
    }
}
impl<'a, Symbol> Error for PipelineError<'a, Symbol> where Symbol: Debug+PartialEq{}

pub type PipelineResult<'a, Symbol> = Result<MatchingPipeline<Symbol>, PipelineError<'a, Symbol>>;

trait Quantifier{}

pub struct Exactly(pub NonZeroUsize); impl Quantifier for Exactly{}
/*struct ZeroOrOne; impl Quantifier for ZeroOrOne{}
struct AtLeast(usize); impl Quantifier for AtLeast{}
struct AtMost(usize); impl Quantifier for AtMost{}*/

pub trait Quantifiable<'a, Q:Quantifier, Symbol>:Sized where Symbol: Debug+PartialEq+Clone{
    fn with_quantifier<F>(self, quantifier:Q, callback: F) -> PipelineResult<'a, Symbol> where F: Fn(Self) -> PipelineResult<'a, Symbol>;
}

impl<'a, Symbol> Quantifiable<'a, Exactly, Symbol> for MatchingPipeline<Symbol> where Symbol: PartialEq+Clone+Debug {
    fn with_quantifier<F>(mut self, quantifier:Exactly, callback: F) -> PipelineResult<'a, Symbol> where F: Fn(Self) -> PipelineResult<'a, Symbol> {

        let mut n:Option<NonZeroUsize> = None;
        loop{

            if self.reached_eos {
                break;
            }

            let pipeline = self.clone();

            let result = callback(pipeline);

            match result {
                Ok(s) => {
                    self = s;
                    if n.is_none() {
                        n = Some(NonZeroUsize::new(1).unwrap())
                    }else{
                        n = n.and_then(|x| x.checked_add(1));
                    }
                    
                    // On est sûr que n vaut Some(...)
                    let n = n.unwrap();

                    if n == quantifier.0 {
                        return Ok(self);
                    }else if n < quantifier.0 {
                        continue;
                    }/*else {
                        return Err(format!("Unexpected: {0:?}", self.unmatched.get(0)));
                    }*/
                },

                Err(_) => return result
            }

        }

        Err(PipelineError::UnexpectedEos)
        
    }
}



impl<'a, Symbol> MatchingPipeline<Symbol> where Symbol: PartialEq+Clone+Debug+Copy{
    fn new(candidate: impl Iterator<Item = Symbol>) -> Self{
        let collection = candidate.collect::<Vec<Symbol>>();
        Self { matched: vec![], reached_eos: collection.is_empty(), unmatched: collection  }
    }

    /// Matches the current symbol:
    /// 
    /// The symbol is added to the list of matched symbols
    /// and the pipeline moves to the next symbol of the sequence
    pub fn consume(mut self) -> Self {
        if self.reached_eos {
            return self;
        }

        let (matched, unmatched) = self.unmatched.split_at(1);

        self.matched.append(&mut matched.to_vec());
        self.unmatched = unmatched.to_vec();

        self.reached_eos = self.unmatched.is_empty();

        self
    }

    /// Moves the pipeline to the next symbol of the sequence
    /// 
    /// The current symbol is not added to the matched symbols list
    pub fn skip(mut self) -> Self {
        if self.reached_eos {
            return self;
        }

        self.unmatched = self.unmatched.get(1..).unwrap_or_default().to_vec();

        self.reached_eos = self.unmatched.is_empty();

        self
    }

    /// Expects that `symbol` can be matched
    /// 
    /// * `symbol` - The expected symbol
    pub fn match_symbol(self, symbol:&'a Symbol) -> PipelineResult<'a, Symbol>{
        if self.reached_eos {
            return Err(PipelineError::UnexpectedEos);
        }

        let actual = self.unmatched.get(0).unwrap().clone();
        if symbol == &actual {
            return  Ok(self.consume());
        }

        Err(PipelineError::WrongSymbol { expected: &symbol, actual})
        
    }

    /// Expects that `pattern` can be matched
    /// 
    /// * `pattern` - The expected pattern
    pub fn match_pattern(mut self, pattern:&'a [Symbol]) -> PipelineResult<'a, Symbol>{
        let pipeline = self.clone();
        match pipeline.unmatched.get(0..pattern.len()) {
            Some(symbols) if symbols == pattern => {
                for symbol in symbols{
                    //Todo: replace with consume()
                    self = self.match_symbol(symbol).expect("All symbols should match");
                }

                Ok(self)
            },

            Some(s) => {
                Err(PipelineError::WrongPattern { expected: pattern, actual: s.to_vec() })
            },

            None => Err(PipelineError::WrongPattern { expected: pattern, actual: pipeline.unmatched.to_vec() })
        }
    }

    /// Expects that `symbols` contains the current symbol 
    /// 
    /// * `symbols` - A list of symbols
    pub fn match_any_of(mut self, symbols:&'a [Symbol]) -> PipelineResult<'a, Symbol> {
        if self.reached_eos {
            return Err(PipelineError::UnexpectedEos);
        }
        let actual = self.unmatched.get(0).unwrap().clone();

        if symbols.contains(&actual) {

            self = self.consume();

            return Ok(self);
        }

        Err(PipelineError::SymbolNotMatchAnyOf { expected: symbols, actual })
    }

    /// Expects that the current symbol is part of [SymbolGroup]
    pub fn match_any_of_group(mut self, group: SymbolGroup<'a, Symbol>) -> PipelineResult<'a, Symbol>{
        if self.reached_eos {
            return Err(PipelineError::UnexpectedEos);
        }

        let actual = self.unmatched.get(0).unwrap().clone();
        if group.accept(&actual) {

            self = self.consume();

            return Ok(self);
        }

        Err(PipelineError::SymbolNotPartOfGroup { expected: group, actual })
    }


    /// Matches all symbols until it matches the pattern `delim` or reaches end of stream
    /// 
    /// `delim` is also matched
    /// 
    /// * `delim` - The delimiter pattern
    pub fn match_until(mut self, delim:&[Symbol]) -> PipelineResult<'a, Symbol> {
    
        loop {
            if self.reached_eos {
                break;
            }

            let pipeline = self.clone();
            
            if let Ok(s) = pipeline.match_pattern(delim){
                self = s;
                break;
            }
            self = self.consume();

            
        }

        Ok(self)
    }

    /// Matches all symbols until one is part of [SymbolGroup] or reaches end of stream
    /// 
    /// The delimiting symbol is also matched
    pub fn match_until_group(mut self, group: &SymbolGroup<'a, Symbol>) -> PipelineResult<'a, Symbol> {
        loop {
            if self.reached_eos {
                break;
            }

            let pipeline = self.clone();
            let symbol = pipeline.unmatched.get(0).expect("Should not be empty");
            if group.accept(symbol) {
                self = pipeline.consume();
                break;
            }
            
            self = pipeline.consume();
        }

        Ok(self)
    }

    /// Encapsulates the logic inside a closure
    pub fn block<F>(self, callback: F) -> PipelineResult<'a, Symbol> where F: Fn(Self) -> PipelineResult<'a, Symbol> {
        callback(self)
    }

    
}


/// Creates a [MatchingPipeline]
pub fn begin_match<U>(candidate: impl Into<MatchingPipeline<U>>) -> MatchingPipeline<U> where U:PartialEq+Clone{
    candidate.into()
}







/*
PatternMatcher
match_xxx => MatchedPattern
let str = "-12.24"

begin_match(str)
    .match_symbol('r')

let (otherDataType, undigestedPattern) = str.match_symbol('-')
    .fail(|| )
    .then()
        .match_digit(Quantifier::OneOrMany)
        .match_symbol('.')
        .match_digit(Quantifier::OneOrMany)
        .digest()
        .map(|digested_pattern| OtherDataType)

match_symbol('-')
    .fail(||)
    .match_digit()
    .is_complete()

*/