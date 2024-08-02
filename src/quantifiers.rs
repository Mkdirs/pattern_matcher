use std::num::NonZeroUsize;

use crate::{MatchingPipeline, PipelineResult, Symbol};

pub trait Quantifier{}

pub struct Exactly(pub NonZeroUsize); impl Quantifier for Exactly{}
pub struct ZeroOrOne; impl Quantifier for ZeroOrOne{}
pub struct AtLeast(pub usize); impl Quantifier for AtLeast{}
pub struct AtMost(pub NonZeroUsize); impl Quantifier for AtMost{}
pub struct ZeroOrMore; impl Quantifier for ZeroOrMore{}

pub trait WithQuantifier<'a, Q:Quantifier, S:Symbol> {
    fn with_quantifier<F>(self, quantifier:Q, callback: F) -> PipelineResult<'a, S> where F: Fn(Self) -> PipelineResult<'a, S>, Self: Sized;
}

impl<'a, S:Symbol> WithQuantifier<'a, Exactly, S> for MatchingPipeline<S>  {
    fn with_quantifier<F>(mut self, quantifier:Exactly, callback: F) -> PipelineResult<'a, S> where F: Fn(Self) -> PipelineResult<'a, S> {

        let mut n:Option<NonZeroUsize> = None;
        loop{

            let pipeline = self.clone();

            let result = pipeline.block(&callback);

            if let Ok(p) = result {
                self = p;

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
                }

            }else{
                return result;
            }

            

        }

        
    }
}

impl<'a, S:Symbol> WithQuantifier<'a, ZeroOrOne, S> for MatchingPipeline<S> {
    fn with_quantifier<F>(mut self, _:ZeroOrOne, callback: F) -> PipelineResult<'a, S> where F: Fn(Self) -> PipelineResult<'a, S>, Self: Sized {
        
        let p = self.clone();
        if let Ok(pipeline) = p.block(callback){
            self = pipeline;
        }

        Ok(self)
    }
}

impl<'a, S:Symbol> WithQuantifier<'a, AtLeast, S> for MatchingPipeline<S> {
    fn with_quantifier<F>(mut self, quantifier:AtLeast, callback: F) -> PipelineResult<'a, S> where F: Fn(Self) -> PipelineResult<'a, S>, Self: Sized {
        let mut n = 0;
        loop {

            let pipeline = self.clone();
            let result = pipeline.block(&callback);

            if let Ok(p) = result {
                self = p;

                n += 1;
            }else if n >= quantifier.0{
                return Ok(self);
            }else {
                return result;
            }
        }
        
    }
}

impl<'a, S:Symbol> WithQuantifier<'a, AtMost, S> for MatchingPipeline<S> {
    fn with_quantifier<F>(mut self, quantifier:AtMost, callback: F) -> PipelineResult<'a, S> where F: Fn(Self) -> PipelineResult<'a, S>, Self: Sized {
        
        let mut  n:Option<NonZeroUsize> = None;

        loop {
            let pipeline = self.clone();
            let result = pipeline.block(&callback);

            if let Ok(p) = result {
                self = p;

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
                }
            }else{

                if n.is_none(){
                    return result;
                }

                let n = n.unwrap();
                if n <= quantifier.0 {
                    return Ok(self);
                }

                return result;
            }
        }
    }
}

impl<'a, S:Symbol> WithQuantifier<'a, ZeroOrMore, S> for MatchingPipeline<S>{
    fn with_quantifier<F>(self, _:ZeroOrMore, callback: F) -> PipelineResult<'a, S> where F: Fn(Self) -> PipelineResult<'a, S>, Self: Sized {
        self.with_quantifier(AtLeast(0), callback)
    }
}