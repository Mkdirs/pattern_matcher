use std::{error::Error, fmt::{Debug, Display}};

#[cfg(test)]
mod tests;

mod quantifiers;
mod digesters;
pub use quantifiers::*;
pub use digesters::*;

pub trait Symbol:PartialEq+Clone+Debug{}

#[derive(Debug, Clone, PartialEq)]
/// This structure helps you build a pattern matching pipeline
pub struct MatchingPipeline<S:Symbol>{
    matched: Vec<S>,
    unmatched: Vec<S>,
    reached_eos: bool,
    offset:usize
}

#[derive(Debug)]
pub struct TerminatedPipeline<S:Symbol>{
    matched:Vec<S>,
    unmatched:Vec<S>,
    offset: usize
}

#[derive(Debug, PartialEq)]
pub enum PipelineError<'a, S:Symbol>{
    UnexpectedEos,
    WrongSymbol{
        expected: &'a S,
        actual: S
    },
    WrongPattern{
        expected: &'a [S],
        actual: Vec<S>
    },


    SymbolNotMatchAnyOf{
        expected: &'a [S],
        actual: S
    },

    SymbolNotMatchingPredicate{actual: S},

    Unexpected{ message: &'a str }

}


impl<'a, S:Symbol> Display for PipelineError<'a, S>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self{
            &Self::UnexpectedEos => write!(f, "Unexpected end of stream"),
            &Self::WrongSymbol { expected, actual } => write!(f, "Expected {expected:?} but instead got {actual:?}"),
            &Self::WrongPattern { expected, actual } => write!(f, "Expected pattern {expected:?} but instead got {actual:?}"),
            &Self::SymbolNotMatchAnyOf { expected, actual } => write!(f, "Expected one of {expected:?} but instead got {actual:?}"),
            &Self::SymbolNotMatchingPredicate { actual } => write!(f, "{actual:?} does not match the given predicate"),
            &Self::Unexpected{message} => write!(f, "Unexpected error: {message}")
        }
    }
}

impl<'a, S:Symbol> Error for PipelineError<'a, S>{}

pub type PipelineResult<'a, Symbol> = Result<MatchingPipeline<Symbol>, PipelineError<'a, Symbol>>;



impl<'a, S:Symbol> MatchingPipeline<S>{
    pub fn new(candidate: impl IntoIterator<Item = S>) -> Self{
        let collection = candidate.into_iter().collect::<Vec<S>>();
        Self { matched: vec![], reached_eos: collection.is_empty(), unmatched: collection, offset: 0  }
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

        if !self.reached_eos {
            self.offset += 1;
        }

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

        if !self.reached_eos {
            self.offset += 1;
        }

        self.reached_eos = self.unmatched.is_empty();

        self
    }

    /// Expects that `symbol` can be matched
    /// 
    /// * `symbol` - The expected symbol
    pub fn expect_symbol(self, symbol:&'a S) -> PipelineResult<'a, S>{
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
    pub fn expect_pattern(mut self, pattern:&'a [S]) -> PipelineResult<'a, S>{
        let pipeline = self.clone();
        match pipeline.unmatched.get(0..pattern.len()) {
            Some(symbols) if symbols == pattern => {
                for symbol in symbols{
                    //Todo: replace with consume()
                    self = self.expect_symbol(symbol).expect("All symbols should match");
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
    pub fn expect_any_of(mut self, symbols:&'a [S]) -> PipelineResult<'a, S> {
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


    /// Matches all symbols until it matches the pattern `delim` or reaches end of stream
    /// 
    /// * `delim` - The delimiter pattern
    /// 
    /// * `match_delim` - If the delimiter is matched or not
    pub fn match_until(mut self, delim:&'a [S], match_delim:bool) -> Self {
    
        loop {
            if self.reached_eos {
                break;
            }

            let pipeline = self.clone();
            
            if let Ok(s) = pipeline.expect_pattern(delim){
                if match_delim { self = s; }
                break;
            }
            self = self.consume();

            
        }

        self
    }

    /// Matches all symbols until it reaches end of stream
    pub fn match_until_eos(mut self) -> Self {
    
        loop {
            if self.reached_eos {
                break;
            }

            self = self.consume();

            
        }

        self
    }

    /// Expects that the current symbol matches the predicate.
    pub fn expect_predicate<F>(mut self, predicate: F) -> PipelineResult<'a, S>
    where F: Fn(&S) -> bool
    {
        if self.reached_eos{
            return Err(PipelineError::UnexpectedEos);
        }

        if predicate(&self.unmatched[0]) {
            self = self.consume();
            return Ok(self);
        }

        Err(PipelineError::SymbolNotMatchingPredicate { actual: self.unmatched[0].clone() })
    }

    /// Matches all symbols until predicate fail or reaches end of stream.
    pub fn match_while_true<F>(mut self, predicate: F) -> Self
    where F: Fn(&S) -> bool
    {
        loop {
            if self.reached_eos {
                break;
            }

            if predicate(&self.unmatched[0]) {
                self = self.consume();
            }else{
                break;
            }
        }

        self
    }

    /// Encapsulates the logic inside a closure
    pub fn block<F>(self, callback: F) -> PipelineResult<'a, S> where F: Fn(Self) -> PipelineResult<'a, S> {
        callback(self)
    }

    pub fn terminate(self) -> TerminatedPipeline<S> {
        TerminatedPipeline{
            unmatched: self.unmatched,
            matched: self.matched,
            offset: self.offset
        }
    }

    
}

impl<S:Symbol> TerminatedPipeline<S>{

    pub fn matched(&self) -> &[S]{
        &self.matched
    }

    pub fn unmatched(&self) -> &[S]{
        &self.unmatched
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn digest<D>(self) -> <D as Digester<S>>::Output
    where D: Digester<S>
    {
        D::digest(&self.matched)
    }
}

pub trait Matchable<S:Symbol>: Into<MatchingPipeline<S>> {}

impl Symbol for char{}

impl<T: AsRef<str>> Matchable<char> for T{}

impl<T: AsRef<str>> From<T> for MatchingPipeline<char>{
    fn from(value: T) -> Self {
        MatchingPipeline::new(value.as_ref().chars())
    }
}





/// Creates a [MatchingPipeline]
pub fn begin_match<S>(candidate: impl Matchable<S>) -> MatchingPipeline<S> where S:Symbol{
    candidate.into()
}

/// A convenient way to tell if a pattern match a pipeline or not
/// while delegating the error handling in a function
pub trait MatchAgainst<'a, S:Symbol+'a, F> : Clone
where F: Fn(MatchingPipeline<S>) -> Result<TerminatedPipeline<S>, PipelineError<'a, S>>
{
    /// Matches a pattern against a pipeline
    /// 
    /// This was meant to relieve the immediate code of the error handling of the pipeline.
    /// It is not recommended to use an anonymous function for callback.
    /// 
    /// Returns Some([TerminatedPipeline]) if successful
    /// 
    /// Returns None if not
    fn match_against(&self, callback: F) -> Option<TerminatedPipeline<S>>;
}

impl<'a, S:Symbol+'a, F, T:Clone> MatchAgainst<'a, S, F> for T
where F: Fn(MatchingPipeline<S>) -> Result<TerminatedPipeline<S>, PipelineError<'a, S>>,
T: Matchable<S>
{
    fn match_against(&self, callback: F) -> Option<TerminatedPipeline<S>> {
        if let Ok(pipeline) = callback(begin_match(self.clone())){
            return Some(pipeline)
        }
        None
    }
}