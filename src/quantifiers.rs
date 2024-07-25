use std::num::NonZeroUsize;

use crate::{MatchingPipeline, PipelineError, PipelineResult, Symbol};

pub trait Quantifier{}

pub struct Exactly(pub NonZeroUsize); impl Quantifier for Exactly{}
/*struct ZeroOrOne; impl Quantifier for ZeroOrOne{}
struct AtLeast(usize); impl Quantifier for AtLeast{}
struct AtMost(usize); impl Quantifier for AtMost{}*/

pub trait WithQuantifier<'a, Q:Quantifier, S:Symbol> {
    fn with_quantifier<F>(self, quantifier:Q, callback: F) -> PipelineResult<'a, S> where F: Fn(Self) -> PipelineResult<'a, S>, Self: Sized;
}

impl<'a, S:Symbol> WithQuantifier<'a, Exactly, S> for MatchingPipeline<S>  {
    fn with_quantifier<F>(mut self, quantifier:Exactly, callback: F) -> PipelineResult<'a, S> where F: Fn(Self) -> PipelineResult<'a, S> {

        let mut n:Option<NonZeroUsize> = None;
        loop{

            if self.reached_eos {
                break;
            }

            let pipeline = self.clone();

            let result = pipeline.block(&callback);

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
