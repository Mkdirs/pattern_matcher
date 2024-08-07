use std::num::NonZeroUsize;

use crate::{begin_match, quantifiers::WithQuantifier, AtLeast, AtMost, Exactly, MatchingPipeline, PipelineError, SymbolGroup, ZeroOrOne};

#[test]
fn should_match_all_symbols() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("hello")
        .match_symbol(&'h')?
        .match_symbol(&'e')?
        .match_symbol(&'l')?
        .match_symbol(&'l')?
        .match_symbol(&'o')?;

    let expected = MatchingPipeline{
        matched: vec!['h', 'e', 'l', 'l', 'o'],
        unmatched: vec![],
        reached_eos: true,
        cursor: -1
    };

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn should_not_match_symbol() -> Result<(), PipelineError<'static, char>> {
    let result = begin_match("Fou")
        .match_symbol(&'F')?
        .match_symbol(&'o')?
        .match_symbol(&'o');

    let expected = Err(PipelineError::WrongSymbol { expected: &'o', actual: 'u' });

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn should_not_reach_eos() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("Foxy")
        .match_symbol(&'F')?
        .match_symbol(&'o')?;

    let expected = MatchingPipeline{
        matched: vec!['F', 'o'],
        unmatched: vec!['x', 'y'],
        reached_eos: false,
        cursor: 2
    };

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn should_match_pattern() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("0x85ADG Header")
        .match_pattern(&['0','x','8','5','A','D','G'])?;

    let expected = MatchingPipeline{
        matched: vec!['0','x','8','5','A','D','G'],
        unmatched: vec![' ','H','e','a','d','e','r'],
        reached_eos: false,
        cursor: 7
    };

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn should_not_match_pattern() {
    let pattern = &['0','x','8','6','A','D','G'];
    let result = begin_match("0x85ADG Header")
        .match_pattern(pattern);

    let expected = Err(PipelineError::WrongPattern { expected: pattern, actual: vec!['0','x','8','5','A','D','G'] });

    assert_eq!(result, expected);
}

#[test]
fn pattern_is_too_big() {
    let pattern = &['0','x','8','6','A','D','G'];
    let result = begin_match("0x")
        .match_pattern(pattern);

    let expected = Err(PipelineError::WrongPattern { expected: pattern, actual: vec!['0', 'x'] });

    assert_eq!(result, expected);
}

#[test]
fn should_match_until_comma() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("Foo,Bar ,baz")
        .match_until(&[','])?;

    let expected = MatchingPipeline{
        matched: vec!['F','o','o',','],
        unmatched: vec!['B','a','r',' ',',','b','a','z'],
        reached_eos: false,
        cursor: 4
    };

    assert_eq!(result, expected);
    
    Ok(())
}

#[test]
fn should_skip() {
    let result = begin_match("Fax")
        .consume()
        .skip()
        .consume();

    let expected = MatchingPipeline{
        matched: vec!['F', 'x'],
        unmatched: vec![],
        reached_eos: true,
        cursor: -1
    };

    assert_eq!(result, expected);

}

#[test]
fn should_match_any() -> Result<(), PipelineError<'static, char>>{
    let digits = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    let result = begin_match("1 2 3")
        .match_any_of(digits)?
        .skip()
        .match_any_of(digits)?
        .skip()
        .match_any_of(digits)?;

    let expected = MatchingPipeline{
        matched: vec!['1', '2', '3'],
        unmatched: vec![],
        reached_eos: true,
        cursor: -1
    };

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn should_match_any_group() -> Result<(), PipelineError<'static, char>>{
    let terminator = SymbolGroup{
        accepted_symbols: &[',', ';'],
        description: "',' or ';'"
    };

    let result = begin_match("a;b,1")
        .skip()
        .match_any_of_group(terminator.clone())?
        .skip()
        .match_any_of_group(terminator)?
        .skip();

    let expected = MatchingPipeline{
        matched: vec![';', ','],
        unmatched: vec![],
        reached_eos: true,
        cursor: -1
    };

    assert_eq!(result, expected);

    Ok(())
        
}

#[test]
fn state_should_be_preserved_in_block() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("abcFoo1Bar2")
        .match_symbol(&'a')?.match_symbol(&'b')?.match_symbol(&'c')?
        .block(|p| {
            p.match_pattern(&['F', 'o', 'o', '1'])?
            .match_symbol(&'B')
        })?.match_symbol(&'a')?;

    let expected = MatchingPipeline{
        matched: vec!['a', 'b', 'c', 'F', 'o', 'o', '1', 'B', 'a'],
        unmatched: vec!['r', '2'],
        reached_eos: false,
        cursor: 9
    };

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn should_match_exactly_3() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("18a18b18c")
    .with_quantifier(Exactly(NonZeroUsize::new(3).unwrap()), |p|{
        p.match_pattern(&['1', '8'])?
        .match_any_of(&['a', 'b', 'c'])
    })?;

    let expected = MatchingPipeline {
        matched: vec!['1','8','a', '1','8','b', '1','8','c'],
        unmatched: vec![],
        reached_eos: true,
        cursor: -1
    };

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn quantifier_exactly_do_not_match() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("abaabaO")
    .with_quantifier(Exactly(NonZeroUsize::new(3).unwrap()), |p| {
        p.match_pattern(&['a', 'b', 'a'])
    });

    let expected = Err(PipelineError::WrongPattern { expected: &['a', 'b', 'a'], actual: vec!['O'] });

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn quantifier_zero_or_one_with_trailing_expectation() -> Result<(), PipelineError<'static, char>>{
    let candidate1 = "abc";
    let candidate2 = "ab";
    let candidate3 = "ac";
    let candidate4 = "axc";
    
    let result1 = begin_match(candidate1)
    .match_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.match_symbol(&'b'))?
    .match_symbol(&'c')?;

    let expected1 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'b', 'c'],
        reached_eos: true,
        cursor: -1
    };

    let result2 = begin_match(candidate2)
    .match_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.match_symbol(&'b'))?
    .match_symbol(&'c');

    let expected2 = Err(PipelineError::UnexpectedEos);

    let result3 = begin_match(candidate3)
    .match_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.match_symbol(&'b'))?
    .match_symbol(&'c')?;

    let expected3 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'c'],
        reached_eos: true,
        cursor: -1
    };

    let result4 = begin_match(candidate4)
    .match_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.match_symbol(&'b'))?
    .match_symbol(&'c');

    let expected4 = Err(PipelineError::WrongSymbol { expected: &'c', actual: 'x' });

    assert_eq!(result1, expected1);
    assert_eq!(result2, expected2);
    assert_eq!(result3, expected3);
    assert_eq!(result4, expected4);

    

    Ok(())
}


#[test]
fn quantifier_zero_or_one_without_trailing_expectation() -> Result<(), PipelineError<'static, char>>{
    let candidate1 = "abc";
    let candidate2 = "ab";
    let candidate3 = "ac";
    let candidate4 = "axc";
    
    let result1 = begin_match(candidate1)
    .match_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.match_symbol(&'b'))?;

    let expected1 = MatchingPipeline{
        unmatched: vec!['c'],
        matched: vec!['a', 'b'],
        reached_eos: false,
        cursor: 2
    };

    let result2 = begin_match(candidate2)
    .match_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.match_symbol(&'b'))?;

    let expected2 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'b'],
        reached_eos: true,
        cursor: -1
    };

    let result3 = begin_match(candidate3)
    .match_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.match_symbol(&'b'))?;

    let expected3 = MatchingPipeline{
        unmatched: vec!['c'],
        matched: vec!['a'],
        reached_eos: false,
        cursor: 1
    };

    let result4 = begin_match(candidate4)
    .match_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.match_symbol(&'b'))?;

    let expected4 = MatchingPipeline{
        unmatched: vec!['x', 'c'],
        matched: vec!['a'],
        reached_eos: false,
        cursor: 1
    };

    assert_eq!(result1, expected1);
    assert_eq!(result2, expected2);
    assert_eq!(result3, expected3);
    assert_eq!(result4, expected4);

    

    Ok(())
}

#[test]
fn quantifier_at_least() -> Result<(), PipelineError<'static, char>> {
    let result1 = begin_match("abbbc")
    .match_symbol(&'a')?
    .with_quantifier(AtLeast(3), |p| p.match_symbol(&'b'))?
    .match_symbol(&'c')?;

    let expected1 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'b', 'b', 'b', 'c'],
        reached_eos: true,
        cursor: -1
    };

    let result2 = begin_match("abb")
    .match_symbol(&'a')?
    .with_quantifier(AtLeast(3), |p| p.match_symbol(&'b'));

    let expected2 = Err(PipelineError::UnexpectedEos);

    let result3 = begin_match("abx")
    .match_symbol(&'a')?
    .with_quantifier(AtLeast(3), |p| p.match_symbol(&'b'));

    let expected3 = Err(PipelineError::WrongSymbol { expected: &'b', actual: 'x' });

    let result4 = begin_match("abbbc")
    .match_symbol(&'a')?
    .with_quantifier(AtLeast(1), |p| p.match_symbol(&'b'))?
    .match_symbol(&'c')?;

    let expected4 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'b', 'b', 'b', 'c'],
        reached_eos: true,
        cursor: -1
    };

    let result5 = begin_match("abbc")
    .match_symbol(&'a')?
    .with_quantifier(AtLeast(0), |p| p.match_symbol(&'b'))?
    .match_symbol(&'c')?;

    let expected5 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'b', 'b', 'c'],
        reached_eos: true,
        cursor: -1
    };

    let result6 = begin_match("ac")
    .match_symbol(&'a')?
    .with_quantifier(AtLeast(0), |p| p.match_symbol(&'b'))?
    .match_symbol(&'c')?;

    let expected6 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'c'],
        reached_eos: true,
        cursor: -1
    };

    assert_eq!(result1, expected1);
    assert_eq!(result2, expected2);
    assert_eq!(result3, expected3);
    assert_eq!(result4, expected4);
    assert_eq!(result5, expected5);
    assert_eq!(result6, expected6);

    Ok(())
}

#[test]
fn quantifier_at_most() -> Result<(), PipelineError<'static, char>>{

    let result1 = begin_match("aab")
    .with_quantifier(AtMost(NonZeroUsize::new(3).unwrap()), |p| p.match_symbol(&'a'))?
    .match_symbol(&'b')?;

    let expected1 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'a', 'b'],
        reached_eos: true,
        cursor: -1
    };

    let result2 = begin_match("aaaax")
    .with_quantifier(AtMost(NonZeroUsize::new(3).unwrap()), |p| p.match_symbol(&'a'))?;

    let expected2 = MatchingPipeline{
        unmatched: vec!['a', 'x'],
        matched: vec!['a', 'a', 'a'],
        reached_eos: false,
        cursor: 3
    };

    assert_eq!(result1, expected1);
    assert_eq!(result2, expected2);

    Ok(())
}