use std::num::NonZeroUsize;

use crate::{begin_match, quantifiers::WithQuantifier, AtLeast, AtMost, Exactly, MatchingPipeline, PipelineError, ZeroOrOne};

#[test]
fn should_match_all_symbols() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("hello")
        .expect_symbol(&'h')?
        .expect_symbol(&'e')?
        .expect_symbol(&'l')?
        .expect_symbol(&'l')?
        .expect_symbol(&'o')?;

    let expected = MatchingPipeline{
        matched: vec!['h', 'e', 'l', 'l', 'o'],
        unmatched: vec![],
        reached_eos: true,
        offset: 5
    };

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn should_not_match_symbol() -> Result<(), PipelineError<'static, char>> {
    let result = begin_match("Fou")
        .expect_symbol(&'F')?
        .expect_symbol(&'o')?
        .expect_symbol(&'o');

    let expected = Err(PipelineError::WrongSymbol { expected: &'o', actual: 'u' });

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn should_not_reach_eos() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("Foxy")
        .expect_symbol(&'F')?
        .expect_symbol(&'o')?;

    let expected = MatchingPipeline{
        matched: vec!['F', 'o'],
        unmatched: vec!['x', 'y'],
        reached_eos: false,
        offset: 2
    };

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn should_match_pattern() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("0x85ADG Header")
        .expect_pattern(&['0','x','8','5','A','D','G'])?;

    let expected = MatchingPipeline{
        matched: vec!['0','x','8','5','A','D','G'],
        unmatched: vec![' ','H','e','a','d','e','r'],
        reached_eos: false,
        offset: 7
    };

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn should_not_match_pattern() {
    let pattern = &['0','x','8','6','A','D','G'];
    let result = begin_match("0x85ADG Header")
        .expect_pattern(pattern);

    let expected = Err(PipelineError::WrongPattern { expected: pattern, actual: vec!['0','x','8','5','A','D','G'] });

    assert_eq!(result, expected);
}

#[test]
fn pattern_is_too_big() {
    let pattern = &['0','x','8','6','A','D','G'];
    let result = begin_match("0x")
        .expect_pattern(pattern);

    let expected = Err(PipelineError::WrongPattern { expected: pattern, actual: vec!['0', 'x'] });

    assert_eq!(result, expected);
}

#[test]
fn should_match_until_comma(){
    let result = begin_match("Foo,Bar ,baz")
        .match_until(&[','], true);

    let expected = MatchingPipeline{
        matched: vec!['F','o','o',','],
        unmatched: vec!['B','a','r',' ',',','b','a','z'],
        reached_eos: false,
        offset: 4
    };

    assert_eq!(result, expected);
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
        offset: 3
    };

    assert_eq!(result, expected);

}

#[test]
fn should_match_any() -> Result<(), PipelineError<'static, char>>{
    let digits = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    let result = begin_match("1 2 3")
        .expect_any_of(digits)?
        .skip()
        .expect_any_of(digits)?
        .skip()
        .expect_any_of(digits)?;

    let expected = MatchingPipeline{
        matched: vec!['1', '2', '3'],
        unmatched: vec![],
        reached_eos: true,
        offset: 5
    };

    assert_eq!(result, expected);

    Ok(())
}


#[test]
fn state_should_be_preserved_in_block() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("abcFoo1Bar2")
        .expect_symbol(&'a')?.expect_symbol(&'b')?.expect_symbol(&'c')?
        .block(|p| {
            p.expect_pattern(&['F', 'o', 'o', '1'])?
            .expect_symbol(&'B')
        })?.expect_symbol(&'a')?;

    let expected = MatchingPipeline{
        matched: vec!['a', 'b', 'c', 'F', 'o', 'o', '1', 'B', 'a'],
        unmatched: vec!['r', '2'],
        reached_eos: false,
        offset: 9
    };

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn should_match_exactly_3() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("18a18b18c")
    .with_quantifier(Exactly(NonZeroUsize::new(3).unwrap()), |p|{
        p.expect_pattern(&['1', '8'])?
        .expect_any_of(&['a', 'b', 'c'])
    })?;

    let expected = MatchingPipeline {
        matched: vec!['1','8','a', '1','8','b', '1','8','c'],
        unmatched: vec![],
        reached_eos: true,
        offset: 9
    };

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn quantifier_exactly_do_not_match() -> Result<(), PipelineError<'static, char>>{
    let result = begin_match("abaabaO")
    .with_quantifier(Exactly(NonZeroUsize::new(3).unwrap()), |p| {
        p.expect_pattern(&['a', 'b', 'a'])
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
    .expect_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.expect_symbol(&'b'))?
    .expect_symbol(&'c')?;

    let expected1 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'b', 'c'],
        reached_eos: true,
        offset: 3
    };

    let result2 = begin_match(candidate2)
    .expect_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.expect_symbol(&'b'))?
    .expect_symbol(&'c');

    let expected2 = Err(PipelineError::UnexpectedEos);

    let result3 = begin_match(candidate3)
    .expect_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.expect_symbol(&'b'))?
    .expect_symbol(&'c')?;

    let expected3 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'c'],
        reached_eos: true,
        offset: 2
    };

    let result4 = begin_match(candidate4)
    .expect_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.expect_symbol(&'b'))?
    .expect_symbol(&'c');

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
    .expect_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.expect_symbol(&'b'))?;

    let expected1 = MatchingPipeline{
        unmatched: vec!['c'],
        matched: vec!['a', 'b'],
        reached_eos: false,
        offset: 2
    };

    let result2 = begin_match(candidate2)
    .expect_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.expect_symbol(&'b'))?;

    let expected2 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'b'],
        reached_eos: true,
        offset: 2
    };

    let result3 = begin_match(candidate3)
    .expect_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.expect_symbol(&'b'))?;

    let expected3 = MatchingPipeline{
        unmatched: vec!['c'],
        matched: vec!['a'],
        reached_eos: false,
        offset: 1
    };

    let result4 = begin_match(candidate4)
    .expect_symbol(&'a')?
    .with_quantifier(ZeroOrOne, |p| p.expect_symbol(&'b'))?;

    let expected4 = MatchingPipeline{
        unmatched: vec!['x', 'c'],
        matched: vec!['a'],
        reached_eos: false,
        offset: 1
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
    .expect_symbol(&'a')?
    .with_quantifier(AtLeast(3), |p| p.expect_symbol(&'b'))?
    .expect_symbol(&'c')?;

    let expected1 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'b', 'b', 'b', 'c'],
        reached_eos: true,
        offset: 5
    };

    let result2 = begin_match("abb")
    .expect_symbol(&'a')?
    .with_quantifier(AtLeast(3), |p| p.expect_symbol(&'b'));

    let expected2 = Err(PipelineError::UnexpectedEos);

    let result3 = begin_match("abx")
    .expect_symbol(&'a')?
    .with_quantifier(AtLeast(3), |p| p.expect_symbol(&'b'));

    let expected3 = Err(PipelineError::WrongSymbol { expected: &'b', actual: 'x' });

    let result4 = begin_match("abbbc")
    .expect_symbol(&'a')?
    .with_quantifier(AtLeast(1), |p| p.expect_symbol(&'b'))?
    .expect_symbol(&'c')?;

    let expected4 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'b', 'b', 'b', 'c'],
        reached_eos: true,
        offset: 5
    };

    let result5 = begin_match("abbc")
    .expect_symbol(&'a')?
    .with_quantifier(AtLeast(0), |p| p.expect_symbol(&'b'))?
    .expect_symbol(&'c')?;

    let expected5 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'b', 'b', 'c'],
        reached_eos: true,
        offset: 4
    };

    let result6 = begin_match("ac")
    .expect_symbol(&'a')?
    .with_quantifier(AtLeast(0), |p| p.expect_symbol(&'b'))?
    .expect_symbol(&'c')?;

    let expected6 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'c'],
        reached_eos: true,
        offset: 2
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
    .with_quantifier(AtMost(NonZeroUsize::new(3).unwrap()), |p| p.expect_symbol(&'a'))?
    .expect_symbol(&'b')?;

    let expected1 = MatchingPipeline{
        unmatched: vec![],
        matched: vec!['a', 'a', 'b'],
        reached_eos: true,
        offset: 3
    };

    let result2 = begin_match("aaaax")
    .with_quantifier(AtMost(NonZeroUsize::new(3).unwrap()), |p| p.expect_symbol(&'a'))?;

    let expected2 = MatchingPipeline{
        unmatched: vec!['a', 'x'],
        matched: vec!['a', 'a', 'a'],
        reached_eos: false,
        offset: 3
    };

    assert_eq!(result1, expected1);
    assert_eq!(result2, expected2);

    Ok(())
}