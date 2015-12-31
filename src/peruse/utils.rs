use parsers::*;

pub fn skip<I: ?Sized,O, P: ParserCombinator<I=I, O=O>, S: ParserCombinator<I=I>>(essential: P, skipped: S) ->
MapParser<I, ChainedParser<MapParser<I, ChainedParser<RepeatParser<S>, P>, O>, RepeatParser<S>>, O>{
    skipped.repeat().then_r(essential).then_l(skipped.repeat())
}