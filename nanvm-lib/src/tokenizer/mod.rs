use std::{collections::VecDeque, mem::take, ops::{Deref, RangeInclusive}};

use crate::{
    big_numbers::{
        self, big_float::BigFloat, big_int::{BigInt, Sign}, big_uint::BigUint
    }, common::{cast::Cast, default::default}, js::js_bigint::{self, add, from_u64, zero, JsBigintMutRef, JsBigintRef}, mem::manager::{Dealloc, Manager}, range_map::{from_one, from_range, merge, merge_list, RangeMap, State}
};

#[derive(Debug)]
pub enum JsonToken<D: Dealloc> {
    String(String),
    Number(f64),
    ObjectBegin,
    ObjectEnd,
    ArrayBegin,
    ArrayEnd,
    Colon,
    Comma,
    Equals,
    Dot,
    ErrorToken(ErrorType),
    BigInt(JsBigintMutRef<D>),
    Id(String),
    NewLine,
    Semicolon,
    OpeningParenthesis,
    ClosingParenthesis,
}

#[derive(Debug, PartialEq)]
pub enum ErrorType {
    UnexpectedCharacter,
    InvalidToken,
    InvalidNumber,
    InvalidHex,
    MissingQuotes,
    CommentClosingExpected,
}

#[derive(Default)]
pub enum TokenizerState<D: Dealloc> {
    #[default]
    Initial,
    ParseId(String),
    ParseString(String),
    ParseEscapeChar(String),
    ParseUnicodeChar(ParseUnicodeCharState),
    ParseMinus,
    ParseZero(Sign),
    ParseInt(JsBigintMutRef<D>),
    ParseFracBegin(JsBigintMutRef<D>),
    ParseFrac(FloatState<D>),
    ParseExpBegin(ExpState<D>),
    ParseExpSign(ExpState<D>),
    ParseExp(ExpState<D>),
    ParseBigInt(JsBigintMutRef<D>),
    ParseNewLine,
    ParseCommentStart,
    ParseSinglelineComment,
    ParseMultilineComment,
    ParseMultilineCommentAsterix,
    ParseOperator(String),
}

impl<D: Dealloc> TokenizerState<D> {
    fn push(self, c: char, maps: &TransitionMaps) -> (Vec<JsonToken<D>>, TokenizerState<D>) {
        match self {
            TokenizerState::Initial => get_next_state((), c, &maps.initial, maps),
            TokenizerState::ParseId(s) => get_next_state(s, c, &maps.id, maps),
            TokenizerState::ParseString(s) => get_next_state(s, c, &maps.string, maps),
            TokenizerState::ParseEscapeChar(s) => get_next_state(s, c, &maps.escape_char, maps),
            TokenizerState::ParseUnicodeChar(s) => get_next_state(s, c, &maps.unicode_char, maps),
            TokenizerState::ParseZero(s) => get_next_state(s, c, &maps.zero, maps),
            TokenizerState::ParseInt(s) => get_next_state(s, c, &maps.int, maps),
            TokenizerState::ParseMinus => get_next_state((), c, &maps.minus, maps),
            TokenizerState::ParseFracBegin(s) => get_next_state(s, c, &maps.frac_begin, maps),
            TokenizerState::ParseFrac(s) => get_next_state(s, c, &maps.frac, maps),
            TokenizerState::ParseExpBegin(s) => get_next_state(s, c, &maps.exp_begin, maps),
            TokenizerState::ParseExpSign(s) | TokenizerState::ParseExp(s) => {
                get_next_state(s, c, &maps.exp, maps)
            }
            TokenizerState::ParseBigInt(s) => get_next_state(s, c, &maps.big_int, maps),
            TokenizerState::ParseNewLine => get_next_state((), c, &maps.new_line, maps),
            TokenizerState::ParseCommentStart => get_next_state((), c, &maps.comment_start, maps),
            TokenizerState::ParseSinglelineComment => {
                get_next_state((), c, &maps.singleline_comment, maps)
            }
            TokenizerState::ParseMultilineComment => {
                get_next_state((), c, &maps.multiline_comment, maps)
            }
            TokenizerState::ParseMultilineCommentAsterix => {
                get_next_state((), c, &maps.multiline_comment_asterix, maps)
            }
            TokenizerState::ParseOperator(s) => get_next_state(s, c, &maps.operator, maps),
        }
    }

    pub fn push_mut(&mut self, c: char, tm: &TransitionMaps) -> Vec<JsonToken<D>> {
        let tokens;
        (tokens, *self) = take(self).push(c, tm);
        tokens
    }

    fn end(self) -> Vec<JsonToken<D>> {
        match self {
            TokenizerState::Initial
            | TokenizerState::ParseNewLine
            | TokenizerState::ParseSinglelineComment => default(),
            TokenizerState::ParseId(s) => [JsonToken::Id(s)].cast(),
            TokenizerState::ParseString(_)
            | TokenizerState::ParseEscapeChar(_)
            | TokenizerState::ParseUnicodeChar(_) => {
                [JsonToken::ErrorToken(ErrorType::MissingQuotes)].cast()
            }
            TokenizerState::ParseCommentStart => {
                [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].cast()
            }
            TokenizerState::ParseMultilineComment
            | TokenizerState::ParseMultilineCommentAsterix => {
                [JsonToken::ErrorToken(ErrorType::CommentClosingExpected)].cast()
            }
            TokenizerState::ParseZero(_) => [JsonToken::Number(default())].cast(),
            TokenizerState::ParseInt(s) => [s.into_token()].cast(),
            TokenizerState::ParseFrac(s) => [s.into_token()].cast(),
            TokenizerState::ParseExp(s) => [s.into_token()].cast(),
            TokenizerState::ParseBigInt(s) => [s.into_big_int_token()].cast(),
            TokenizerState::ParseMinus
            | TokenizerState::ParseFracBegin(_)
            | TokenizerState::ParseExpBegin(_)
            | TokenizerState::ParseExpSign(_) => {
                [JsonToken::ErrorToken(ErrorType::InvalidNumber)].cast()
            }
            TokenizerState::ParseOperator(s) => [operator_to_token(s).unwrap()].cast(),
        }
    }
}

pub struct ParseUnicodeCharState {
    s: String,
    unicode: u32,
    index: u8,
}

impl ParseUnicodeCharState {
    fn push<D: Dealloc>(mut self, i: u32) -> (Vec<JsonToken<D>>, TokenizerState<D>) {
        let new_unicode = self.unicode | (i << ((3 - self.index) * 4));
        match self.index {
            3 => {
                let c = char::from_u32(new_unicode);
                match c {
                    Some(c) => continue_string_state(self.s, c),
                    None => (
                        [JsonToken::ErrorToken(ErrorType::InvalidHex)].cast(),
                        TokenizerState::Initial,
                    ),
                }
            }
            0..=2 => {
                self.unicode = new_unicode;
                self.index += 1;
                (default(), TokenizerState::ParseUnicodeChar(self))
            }
            _ => unreachable!(),
        }
    }
}

pub fn bigfloat_to_f64(bf_10: BigFloat<10>) -> f64 {
    let bf_2 = bf_10.to_bin(54);
    bf_2.to_f64()
}

impl BigUint {
    fn add_digit(mut self, c: char) -> BigUint {
        self = &(&self * &BigUint::from_u64(10)) + &BigUint::from_u64(digit_to_number(c));
        self
    }
}

impl<D: Dealloc> JsBigintMutRef<D> {
    fn from_digit<M: Manager>(m: M, sign: js_bigint::Sign, c: char) -> JsBigintMutRef<M::Dealloc> {
        from_u64(m, sign, digit_to_number(c))
    }

    fn add_digit<M: Manager>(self, m: M, c: char) -> JsBigintMutRef<M::Dealloc> {
        add(m, self.deref(), Self::from_digit(m, js_bigint::Sign::Positive, c).deref())
    }

    fn into_float_state(self) -> FloatState<D> {
        FloatState {
            b: self,
            fe: 0,
        }
    }

    fn into_exp_state(self) -> ExpState<D> {
        ExpState {
            b: self,
            fe: 0,
            es: Sign::Positive,
            e: 0,
        }
    }

    fn to_old_bigint(self) -> BigInt {
        let deref = self.deref();
        let sign = match deref.sign() {
            js_bigint::Sign::Positive => big_numbers::big_int::Sign::Positive,
            js_bigint::Sign::Negative => big_numbers::big_int::Sign::Negative,
        };
        BigInt { sign, value: BigUint { value: deref.items().to_vec() }}
    }

    fn into_token(self) -> JsonToken<D> {        
        JsonToken::Number(bigfloat_to_f64(BigFloat {
            significand: self.to_old_bigint(),
            exp: 0,
            non_zero_reminder: false,
        }))
    }

    fn into_big_int_token(self) -> JsonToken<D> {
        JsonToken::BigInt(self)
    }
}

pub struct FloatState<D: Dealloc> {    
    b: JsBigintMutRef<D>,
    fe: i64,
}

impl<D: Dealloc> FloatState<D> {
    fn add_digit<M: Manager>(mut self, m: M, c: char) -> FloatState<M::Dealloc> {
        self.b = self.b.add_digit(m, c);
        self.fe -= 1;
        self
    }

    fn into_exp_state(self) -> ExpState<D> {
        ExpState {
            b: self.b,
            fe: self.fe,
            es: Sign::Positive,
            e: 0,
        }
    }

    fn into_token(self) -> JsonToken<D> {
        JsonToken::Number(bigfloat_to_f64(BigFloat {
            significand: BigInt {
                sign: self.s,
                value: self.b,
            },
            exp: self.fe,
            non_zero_reminder: false,
        }))
    }
}

pub struct ExpState<D: Dealloc> {
    b: JsBigintMutRef<D>,
    fe: i64,
    es: Sign,
    e: i64,
}

impl<D: Dealloc> ExpState<D> {
    const fn add_digit(mut self, c: char) -> ExpState<D> {
        self.e = self.e * 10 + digit_to_number(c) as i64;
        self
    }

    fn into_token(self) -> JsonToken<D> {
        let exp = self.fe
            + match self.es {
                Sign::Positive => self.e,
                Sign::Negative => -self.e,
            };
        JsonToken::Number(bigfloat_to_f64(BigFloat {
            significand: BigInt {
                sign: self.s,
                value: self.b,
            },
            exp,
            non_zero_reminder: false,
        }))
    }
}

const CP_0: u32 = 0x30;

fn operator_to_token<D: Dealloc>(s: String) -> Option<JsonToken<D>> {
    match s.as_str() {
        "{" => Some(JsonToken::ObjectBegin),
        "}" => Some(JsonToken::ObjectEnd),
        "[" => Some(JsonToken::ArrayBegin),
        "]" => Some(JsonToken::ArrayEnd),
        ":" => Some(JsonToken::Colon),
        "," => Some(JsonToken::Comma),
        "=" => Some(JsonToken::Equals),
        "." => Some(JsonToken::Dot),
        ";" => Some(JsonToken::Semicolon),
        "(" => Some(JsonToken::OpeningParenthesis),
        ")" => Some(JsonToken::ClosingParenthesis),
        _ => None,
    }
}

const WHITE_SPACE_CHARS: [char; 4] = [' ', '\n', '\t', '\r'];
const OPERATOR_CHARS: [char; 10] = ['{', '}', '[', ']', ':', ',', '=', ';', '(', ')'];

fn id_start() -> Vec<RangeInclusive<char>> {
    ['a'..='z', 'A'..='Z', one('_'), one('$')].cast()
}

fn id_char() -> Vec<RangeInclusive<char>> {
    ['a'..='z', 'A'..='Z', one('_'), one('$'), '0'..='9'].cast()
}

fn operator_chars_with_dot() -> Vec<RangeInclusive<char>> {
    let c = OPERATOR_CHARS.into_iter().chain(['.']);
    set(c)
}

fn terminal_for_number() -> Vec<RangeInclusive<char>> {
    let c = WHITE_SPACE_CHARS
        .into_iter()
        .chain(OPERATOR_CHARS)
        .chain(['"', '/']);
    set(c)
}

const fn digit_to_number(c: char) -> u64 {
    c as u64 - CP_0 as u64
}

fn start_number(s: Sign, c: char) -> IntegerState {
    IntegerState::from_difit(s, c)
}

fn create_range_map<T>(
    list: Vec<RangeInclusive<char>>,
    t: Transition<T>,
) -> RangeMap<char, State<Transition<T>>> {
    let mut result = RangeMap { list: default() };
    for range in list {
        result = merge(from_range(range, t), result);
    }
    result
}

fn one(c: char) -> RangeInclusive<char> {
    RangeInclusive::new(c, c)
}

fn set(arr: impl IntoIterator<Item = char>) -> Vec<RangeInclusive<char>> {
    let mut result = Vec::new();
    for c in arr {
        result.push(one(c));
    }
    result
}

type Transition<T, D> =
    fn(state: T, c: char, maps: &TransitionMaps) -> (Vec<JsonToken<D>>, TokenizerState<D>);

struct TransitionMap<T, D: Dealloc> {
    def: Transition<T, D>,
    rm: RangeMap<char, State<Transition<T, D>>>,
}

pub struct TransitionMaps<D: Dealloc> {
    initial: TransitionMap<(), D>,
    id: TransitionMap<String, D>,
    string: TransitionMap<String, D>,
    escape_char: TransitionMap<String, D>,
    unicode_char: TransitionMap<ParseUnicodeCharState, D>,
    zero: TransitionMap<Sign, D>,
    int: TransitionMap<JsBigintMutRef<D>, D>,
    minus: TransitionMap<(), D>,
    frac_begin: TransitionMap<JsBigintMutRef<D>, D>,
    frac: TransitionMap<FloatState<D>, D>,
    exp_begin: TransitionMap<ExpState<D>, D>,
    exp: TransitionMap<ExpState<D>, D>,
    big_int: TransitionMap<JsBigintMutRef<D>, D>,
    new_line: TransitionMap<(), D>,
    comment_start: TransitionMap<(), D>,
    singleline_comment: TransitionMap<(), D>,
    multiline_comment: TransitionMap<(), D>,
    multiline_comment_asterix: TransitionMap<(), D>,
    operator: TransitionMap<String, D>,
}

pub fn create_transition_maps<D: Dealloc>() -> TransitionMaps<D> {
    TransitionMaps {
        initial: create_initial_transitions(),
        id: create_id_transitions(),
        string: create_string_transactions(),
        escape_char: create_escape_char_transactions(),
        unicode_char: create_unicode_char_transactions(),
        zero: create_zero_transactions(),
        int: create_int_transactions(),
        minus: create_minus_transactions(),
        frac_begin: create_frac_begin_transactions(),
        frac: create_frac_transactions(),
        exp_begin: create_exp_begin_transactions(),
        exp: create_exp_transactions(),
        big_int: create_big_int_transactions(),
        new_line: create_new_line_transactions(),
        comment_start: create_comment_start_transactions(),
        singleline_comment: create_singleline_comment_transactions(),
        multiline_comment: create_multiline_comment_transactions(),
        multiline_comment_asterix: create_multiline_comment_asterix_transactions(),
        operator: create_operator_transactions(),
    }
}

fn get_next_state<T, D: Dealloc>(
    state: T,
    c: char,
    tm: &TransitionMap<T>,
    maps: &TransitionMaps,
) -> (Vec<JsonToken<D>>, TokenizerState<D>)
where
    T: 'static,
{
    let entry = tm.rm.get(c);
    match &entry.value {
        Some(f) => f(state, c, maps),
        None => (tm.def)(state, c, maps),
    }
}

fn create_initial_transitions() -> TransitionMap<()> {
    type Func = fn(s: (), c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, _, _| {
            (
                [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].cast(),
                TokenizerState::Initial,
            )
        }) as Func,
        rm: merge_list(
            [
                create_range_map(operator_chars_with_dot(), |_, c, _| {
                    (default(), TokenizerState::ParseOperator(c.to_string()))
                }),
                from_range('1'..='9', |_, c, _| {
                    (
                        default(),
                        TokenizerState::ParseInt(start_number(Sign::Positive, c)),
                    )
                }),
                from_one('"', |_, _, _| {
                    (default(), TokenizerState::ParseString(String::default()))
                }),
                from_one('0', |_, _, _| {
                    (default(), TokenizerState::ParseZero(Sign::Positive))
                }),
                from_one('-', |_, _, _| (default(), TokenizerState::ParseMinus)),
                create_range_map(id_start(), |_, c, _| {
                    (default(), TokenizerState::ParseId(c.to_string()))
                }),
                from_one('\n', |_, _, _| (default(), TokenizerState::ParseNewLine)),
                create_range_map(set([' ', '\t', '\r']), |_, _, _| {
                    (default(), TokenizerState::Initial)
                }),
                from_one('/', |_, _, _| {
                    (default(), TokenizerState::ParseCommentStart)
                }),
            ]
            .cast(),
        ),
    }
}

fn create_id_transitions() -> TransitionMap<String> {
    TransitionMap {
        def: |s, c, maps| {
            transfer_state([JsonToken::Id(s)].cast(), TokenizerState::Initial, c, maps)
        },
        rm: create_range_map(id_char(), |mut s, c, _| {
            s.push(c);
            (default(), TokenizerState::ParseId(s))
        }),
    }
}

fn create_string_transactions() -> TransitionMap<String> {
    TransitionMap {
        def: |mut s, c, _| {
            s.push(c);
            (default(), TokenizerState::ParseString(s))
        },
        rm: merge(
            from_one('"', |s, _, _| {
                ([JsonToken::String(s)].cast(), TokenizerState::Initial)
            }),
            from_one('\\', |s, _, _| {
                (default(), TokenizerState::ParseEscapeChar(s))
            }),
        ),
    }
}

fn continue_string_state<D: Dealloc>(mut s: String, c: char) -> (Vec<JsonToken<D>>, TokenizerState<D>) {
    s.push(c);
    (default(), TokenizerState::ParseString(s))
}

fn transfer_state<D: Dealloc>(
    mut vec: Vec<JsonToken<D>>,
    mut state: TokenizerState<D>,
    c: char,
    maps: &TransitionMaps<D>,
) -> (Vec<JsonToken<D>>, TokenizerState<D>) {
    let next_tokens;
    (next_tokens, state) = state.push(c, maps);
    vec.extend(next_tokens);
    (vec, state)
}

fn create_escape_char_transactions<D: Dealloc>() -> TransitionMap<String, D> {
    TransitionMap {
        def: |s, c, maps| {
            transfer_state(
                [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].cast(),
                TokenizerState::ParseString(s),
                c,
                maps,
            )
        },
        rm: merge_list(
            [
                create_range_map(set(['\"', '\\', '/']), |s, c, _| {
                    continue_string_state(s, c)
                }),
                from_one('b', |s, _, _| continue_string_state(s, '\u{8}')),
                from_one('f', |s, _, _| continue_string_state(s, '\u{c}')),
                from_one('n', |s, _, _| continue_string_state(s, '\n')),
                from_one('r', |s, _, _| continue_string_state(s, '\r')),
                from_one('t', |s, _, _| continue_string_state(s, '\t')),
                from_one('u', |s, _, _| {
                    (
                        default(),
                        TokenizerState::ParseUnicodeChar(ParseUnicodeCharState {
                            s,
                            unicode: 0,
                            index: 0,
                        }),
                    )
                }),
            ]
            .cast(),
        ),
    }
}

fn create_unicode_char_transactions<D: Dealloc>() -> TransitionMap<ParseUnicodeCharState, D> {
    type Func = fn(
        state: ParseUnicodeCharState,
        c: char,
        maps: &TransitionMaps<D>,
    ) -> (Vec<JsonToken<D>>, TokenizerState<D>);
    TransitionMap {
        def: |state, c, maps| {
            transfer_state(
                [JsonToken::ErrorToken(ErrorType::InvalidHex)].cast(),
                TokenizerState::ParseString(state.s),
                c,
                maps,
            )
        },
        rm: merge_list(
            [
                from_range(
                    '0'..='9',
                    (|state, c, _| state.push(c as u32 - '0' as u32)) as Func,
                ),
                from_range(
                    'a'..='f',
                    (|state, c, _| state.push(c as u32 - ('a' as u32 - 10))) as Func,
                ),
                from_range(
                    'A'..='F',
                    (|state, c, _| state.push(c as u32 - ('A' as u32 - 10))) as Func,
                ),
            ]
            .cast(),
        ),
    }
}

fn create_zero_transactions<M: Manager>(m: M) -> TransitionMap<Sign, M::Dealloc> {
    type Func = fn(s: Sign, c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, c, maps| tokenize_invalid_number(c, maps)) as Func,
        rm: merge_list(
            [
                from_one(
                    '.',
                    (|s, _, _| {
                        (
                            default(),
                            TokenizerState::ParseFracBegin(zero(m)),
                        )
                    }) as Func,
                ),
                create_range_map(set(['e', 'E']), |s, _, _| {
                    (
                        default(),
                        TokenizerState::ParseExpBegin(ExpState {
                            b: zero(m),
                            fe: 0,
                            es: Sign::Positive,
                            e: 0,
                        }),
                    )
                }),
                from_one('n', |s, _, _| {
                    (
                        default(),
                        TokenizerState::ParseBigInt(zero(m)),
                    )
                }),
                create_range_map(terminal_for_number(), |_, c, maps| {
                    transfer_state(
                        [JsonToken::Number(default())].cast(),
                        TokenizerState::Initial,
                        c,
                        maps,
                    )
                }),
            ]
            .cast(),
        ),
    }
}

fn create_int_transactions<D: Dealloc>() -> TransitionMap<JsBigintMutRef<D>, D> {
    type Func =
        fn(s: JsBigintMutRef<D>, c: char, maps: &TransitionMaps) -> (Vec<JsonToken<D>>, TokenizerState<D>);
    TransitionMap {
        def: (|_, c, maps| tokenize_invalid_number(c, maps)) as Func,
        rm: merge_list(
            [
                from_range(
                    '0'..='9',
                    (|s, c, _| (default(), TokenizerState::ParseInt(s.add_digit(c)))) as Func,
                ),
                from_one('.', |s, _, _| {
                    (default(), TokenizerState::ParseFracBegin(s))
                }),
                create_range_map(set(['e', 'E']), |s, _, _| {
                    (default(), TokenizerState::ParseExpBegin(s.into_exp_state()))
                }),
                from_one('n', |s, _, _| (default(), TokenizerState::ParseBigInt(s))),
                create_range_map(terminal_for_number(), |s, c, maps| {
                    transfer_state([s.into_token()].cast(), TokenizerState::Initial, c, maps)
                }),
            ]
            .cast(),
        ),
    }
}

fn create_frac_begin_transactions<D: Dealloc>() -> TransitionMap<JsBigintMutRef<D>, D> {
    type Func =
        fn(s: IntegerState, c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, c, maps| tokenize_invalid_number(c, maps)) as Func,
        rm: from_range(
            '0'..='9',
            (|s, c, _| {
                (
                    default(),
                    TokenizerState::ParseFrac(s.into_float_state().add_digit(c)),
                )
            }) as Func,
        ),
    }
}

fn create_frac_transactions<D: Dealloc>() -> TransitionMap<FloatState<D>, D> {
    type Func =
        fn(s: FloatState, c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, c, maps| tokenize_invalid_number(c, maps)) as Func,
        rm: merge_list(
            [
                from_range(
                    '0'..='9',
                    (|s, c, _| (default(), TokenizerState::ParseFrac(s.add_digit(c)))) as Func,
                ),
                create_range_map(set(['e', 'E']), |s, _, _| {
                    (default(), TokenizerState::ParseExpBegin(s.into_exp_state()))
                }),
                create_range_map(terminal_for_number(), |s, c, maps| {
                    transfer_state([s.into_token()].cast(), TokenizerState::Initial, c, maps)
                }),
            ]
            .cast(),
        ),
    }
}

fn create_minus_transactions<D: Dealloc>() -> TransitionMap<(), D> {
    type Func = fn(s: (), c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, c, maps| tokenize_invalid_number(c, maps)) as Func,
        rm: merge(
            from_one(
                '0',
                (|_, _, _| (default(), TokenizerState::ParseZero(Sign::Negative))) as Func,
            ),
            from_range('1'..='9', |_, c, _| {
                (
                    default(),
                    TokenizerState::ParseInt(start_number(Sign::Negative, c)),
                )
            }),
        ),
    }
}

fn create_exp_begin_transactions<D: Dealloc>() -> TransitionMap<ExpState<D>, D> {
    type Func = fn(s: ExpState, c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, c, maps| tokenize_invalid_number(c, maps)) as Func,
        rm: merge_list(
            [
                from_range(
                    '0'..='9',
                    (|s, c, _| (default(), TokenizerState::ParseExp(s.add_digit(c)))) as Func,
                ),
                from_one('+', |s, _, _| (default(), TokenizerState::ParseExpSign(s))),
                from_one('-', |mut s, _, _| {
                    (default(), {
                        s.es = Sign::Negative;
                        TokenizerState::ParseExpSign(s)
                    })
                }),
                create_range_map(terminal_for_number(), |s, c, maps| {
                    transfer_state([s.into_token()].cast(), TokenizerState::Initial, c, maps)
                }),
            ]
            .cast(),
        ),
    }
}

fn create_exp_transactions<D: Dealloc>() -> TransitionMap<ExpState<D>, D> {
    type Func = fn(s: ExpState, c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, c, maps| tokenize_invalid_number(c, maps)) as Func,
        rm: merge(
            from_range(
                '0'..='9',
                (|s, c, _| (default(), TokenizerState::ParseExp(s.add_digit(c)))) as Func,
            ),
            create_range_map(terminal_for_number(), |s, c, maps| {
                transfer_state([s.into_token()].cast(), TokenizerState::Initial, c, maps)
            }),
        ),
    }
}

fn create_big_int_transactions<D: Dealloc>() -> TransitionMap<JsBigintMutRef<D>, D> {
    type Func =
        fn(s: IntegerState, c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, c, maps| tokenize_invalid_number(c, maps)) as Func,
        rm: create_range_map(terminal_for_number(), |s, c, maps| {
            transfer_state([s.into_token()].cast(), TokenizerState::Initial, c, maps)
        }),
    }
}

fn tokenize_invalid_number<D: Dealloc>(c: char, maps: &TransitionMaps<D>) -> (Vec<JsonToken<D>>, TokenizerState<D>) {
    transfer_state(
        [JsonToken::ErrorToken(ErrorType::InvalidNumber)].cast(),
        TokenizerState::Initial,
        c,
        maps,
    )
}

fn create_new_line_transactions<D: Dealloc>() -> TransitionMap<(), D> {
    type Func = fn(s: (), c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, c, maps| {
            transfer_state(
                [JsonToken::NewLine].cast(),
                TokenizerState::Initial,
                c,
                maps,
            )
        }) as Func,
        rm: create_range_map(set(WHITE_SPACE_CHARS), |_, _, _| {
            (default(), TokenizerState::ParseNewLine)
        }),
    }
}

fn create_comment_start_transactions<D: Dealloc>() -> TransitionMap<(), D> {
    type Func = fn(s: (), c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, _, _| {
            (
                [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].cast(),
                TokenizerState::Initial,
            )
        }) as Func,
        rm: merge(
            from_one('/', |_, _, _| {
                (default(), TokenizerState::ParseSinglelineComment)
            }),
            from_one('*', |_, _, _| {
                (default(), TokenizerState::ParseMultilineComment)
            }),
        ),
    }
}

fn create_singleline_comment_transactions<D: Dealloc>() -> TransitionMap<(), D> {
    type Func = fn(s: (), c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, _, _| (default(), TokenizerState::ParseSinglelineComment)) as Func,
        rm: create_range_map(set(WHITE_SPACE_CHARS), |_, _, _| {
            (default(), TokenizerState::ParseNewLine)
        }),
    }
}

fn create_multiline_comment_transactions<D: Dealloc>() -> TransitionMap<(), D> {
    type Func = fn(s: (), c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, _, _| (default(), TokenizerState::ParseMultilineComment)) as Func,
        rm: from_one('*', |_, _, _| {
            (default(), TokenizerState::ParseMultilineCommentAsterix)
        }),
    }
}

fn create_multiline_comment_asterix_transactions<D: Dealloc>() -> TransitionMap<(), D> {
    type Func = fn(s: (), c: char, maps: &TransitionMaps) -> (Vec<JsonToken>, TokenizerState);
    TransitionMap {
        def: (|_, _, _| (default(), TokenizerState::ParseMultilineComment)) as Func,
        rm: merge(
            from_one('/', |_, _, _| (default(), TokenizerState::Initial)),
            from_one('*', |_, _, _| {
                (default(), TokenizerState::ParseMultilineCommentAsterix)
            }),
        ),
    }
}

fn create_operator_transactions<D: Dealloc>() -> TransitionMap<String, D> {
    TransitionMap {
        def: |s, c, maps| {
            let token = operator_to_token(s).unwrap();
            transfer_state([token].cast(), TokenizerState::Initial, c, maps)
        },
        rm: create_range_map(operator_chars_with_dot(), |s, c, maps| {
            let mut next_string = s.clone();
            next_string.push(c);
            match operator_to_token(next_string) {
                Some(_) => {
                    let mut next_string = s.clone();
                    next_string.push(c);
                    (default(), TokenizerState::ParseOperator(next_string))
                }
                _ => {
                    let token = operator_to_token(s).unwrap();
                    transfer_state([token].cast(), TokenizerState::Initial, c, maps)
                }
            }
        }),
    }
}

pub fn tokenize<D: Dealloc>(input: String) -> Vec<JsonToken<D>> {
    TokenizerStateIterator::new(input.chars()).collect()
}

pub struct TokenizerStateIterator<T: Iterator<Item = char>, D: Dealloc> {
    chars: T,
    cache: VecDeque<JsonToken<D>>,
    state: TokenizerState<D>,
    maps: TransitionMaps<D>,
    end: bool,
}

impl<T: Iterator<Item = char>, D: Dealloc> TokenizerStateIterator<T, D> {
    pub fn new(chars: T) -> Self {
        Self {
            chars,
            cache: default(),
            state: default(),
            maps: create_transition_maps(),
            end: false,
        }
    }
}

impl<T: Iterator<Item = char>, D: Dealloc> Iterator for TokenizerStateIterator<T, D> {
    type Item = JsonToken<D>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(result) = self.cache.pop_front() {
                return Some(result);
            }
            if self.end {
                return None;
            }
            match self.chars.next() {
                Some(c) => self.cache.extend(self.state.push_mut(c, &self.maps)),
                None => {
                    self.end = true;
                    self.cache.extend(take(&mut self.state).end())
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        big_numbers::{
            big_float::BigFloat,
            big_int::{BigInt, Sign},
            big_uint::BigUint,
        },
        common::cast::Cast,
        tokenizer::bigfloat_to_f64,
    };

    use super::{tokenize, ErrorType, JsonToken};

    #[test]
    #[wasm_bindgen_test]
    fn test_empty() {
        let result = tokenize(String::from(""));
        assert_eq!(result.len(), 0);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_ops() {
        let result = tokenize(String::from("{"));
        assert_eq!(&result, &[JsonToken::ObjectBegin]);

        let result = tokenize(String::from("}"));
        assert_eq!(&result, &[JsonToken::ObjectEnd]);

        let result = tokenize(String::from("["));
        assert_eq!(&result, &[JsonToken::ArrayBegin]);

        let result = tokenize(String::from("]"));
        assert_eq!(&result, &[JsonToken::ArrayEnd]);

        let result = tokenize(String::from(":"));
        assert_eq!(&result, &[JsonToken::Colon]);

        let result = tokenize(String::from(","));
        assert_eq!(&result, &[JsonToken::Comma]);

        let result = tokenize(String::from("="));
        assert_eq!(&result, &[JsonToken::Equals]);

        let result = tokenize(String::from("."));
        assert_eq!(&result, &[JsonToken::Dot]);

        let result = tokenize(String::from(";"));
        assert_eq!(&result, &[JsonToken::Semicolon]);

        let result = tokenize(String::from("()"));
        assert_eq!(
            &result,
            &[JsonToken::OpeningParenthesis, JsonToken::ClosingParenthesis]
        );

        let result = tokenize(String::from("[{ :, }]"));
        assert_eq!(
            &result,
            &[
                JsonToken::ArrayBegin,
                JsonToken::ObjectBegin,
                JsonToken::Colon,
                JsonToken::Comma,
                JsonToken::ObjectEnd,
                JsonToken::ArrayEnd
            ]
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_id() {
        let result = tokenize(String::from("true"));
        assert_eq!(&result, &[JsonToken::Id(String::from("true"))]);

        let result = tokenize(String::from("false"));
        assert_eq!(&result, &[JsonToken::Id(String::from("false"))]);

        let result = tokenize(String::from("null"));
        assert_eq!(&result, &[JsonToken::Id(String::from("null"))]);

        let result = tokenize(String::from("tru tru"));
        assert_eq!(
            &result,
            &[
                JsonToken::Id(String::from("tru")),
                JsonToken::Id(String::from("tru")),
            ]
        );

        let result = tokenize(String::from("ABCxyz_0123456789$"));
        assert_eq!(
            &result,
            &[JsonToken::Id(String::from("ABCxyz_0123456789$")),]
        );

        let result = tokenize(String::from("_"));
        assert_eq!(&result, &[JsonToken::Id(String::from("_")),]);

        let result = tokenize(String::from("$"));
        assert_eq!(&result, &[JsonToken::Id(String::from("$")),]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_whitespace() {
        let result = tokenize(String::from(" \t\n\r"));
        assert_eq!(&result, &[]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_string() {
        let result = tokenize(String::from("\"\""));
        assert_eq!(&result, &[JsonToken::String("".to_string())]);

        let result = tokenize(String::from("\"value\""));
        assert_eq!(&result, &[JsonToken::String("value".to_string())]);

        let result = tokenize(String::from("\"value1\" \"value2\""));
        assert_eq!(
            &result,
            &[
                JsonToken::String("value1".to_string()),
                JsonToken::String("value2".to_string())
            ]
        );

        let result = tokenize(String::from("\"value"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::MissingQuotes)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_escaped_characters() {
        let result = tokenize(String::from("\"\\b\\f\\n\\r\\t\""));
        assert_eq!(
            &result,
            &[JsonToken::String("\u{8}\u{c}\n\r\t".to_string())]
        );

        let result = tokenize(String::from("\"\\x\""));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::UnexpectedCharacter),
                JsonToken::String("x".to_string())
            ]
        );

        let result = tokenize(String::from("\"\\"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::MissingQuotes)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_unicode() {
        let result = tokenize(String::from("\"\\u1234\""));
        assert_eq!(&result, &[JsonToken::String("ሴ".to_string())]);

        let result = tokenize(String::from("\"\\uaBcDEeFf\""));
        assert_eq!(&result, &[JsonToken::String("ꯍEeFf".to_string())]);

        let result = tokenize(String::from("\"\\uEeFg\""));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidHex),
                JsonToken::String("g".to_string())
            ]
        );

        let result = tokenize(String::from("\"\\uEeF"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::MissingQuotes)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_integer() {
        let result = tokenize(String::from("0"));
        assert_eq!(&result, &[JsonToken::Number(0.0)]);

        let result = tokenize(String::from("-0"));
        assert_eq!(&result, &[JsonToken::Number(0.0)]);

        let result = tokenize(String::from("0abc"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Id(String::from("abc"))
            ]
        );

        let result = tokenize(String::from("0. 2"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Number(2.0)
            ]
        );

        let result = tokenize(String::from("1234567890"));
        assert_eq!(&result, &[JsonToken::Number(1234567890.0)]);

        let result = tokenize(String::from("-1234567890"));
        assert_eq!(&result, &[JsonToken::Number(-1234567890.0)]);

        let result = tokenize(String::from("[0,1]"));
        assert_eq!(
            &result,
            &[
                JsonToken::ArrayBegin,
                JsonToken::Number(0.0),
                JsonToken::Comma,
                JsonToken::Number(1.0),
                JsonToken::ArrayEnd
            ]
        );

        let result = tokenize(String::from("001"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Number(1.0),
            ]
        );

        let result = tokenize(String::from("-"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);

        let result = tokenize(String::from("-{}"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::ObjectBegin,
                JsonToken::ObjectEnd
            ]
        );

        let result = tokenize(String::from("9007199254740991"));
        assert_eq!(&result, &[JsonToken::Number(9007199254740991.0)]);

        let result = tokenize(String::from("9007199254740992"));
        assert_eq!(&result, &[JsonToken::Number(9007199254740992.0)]);

        let result = tokenize(String::from("9007199254740993"));
        assert_eq!(&result, &[JsonToken::Number(9007199254740993.0)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_big_float() {
        let result = tokenize(String::from("340282366920938463463374607431768211456"));
        assert_eq!(
            &result,
            &[JsonToken::Number(bigfloat_to_f64(BigFloat {
                significand: BigInt {
                    sign: Sign::Positive,
                    value: BigUint {
                        value: [0, 0, 1].cast()
                    }
                },
                exp: 0,
                non_zero_reminder: false
            }))]
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_float() {
        let result = tokenize(String::from("0.01"));
        assert_eq!(&result, &[JsonToken::Number(0.01)]);

        let result = tokenize(String::from("[-12.34]"));
        assert_eq!(
            &result,
            &[
                JsonToken::ArrayBegin,
                JsonToken::Number(-12.34),
                JsonToken::ArrayEnd
            ]
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_infinity() {
        let result = tokenize(String::from("1e1000"));
        assert_eq!(&result, &[JsonToken::Number(f64::INFINITY)]);

        let result = tokenize(String::from("-1e+1000"));
        assert_eq!(&result, &[JsonToken::Number(f64::NEG_INFINITY)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_exp() {
        let result = tokenize(String::from("1e2"));
        assert_eq!(&result, &[JsonToken::Number(1e2)]);

        let result = tokenize(String::from("1E+2"));
        assert_eq!(&result, &[JsonToken::Number(1e2)]);

        let result = tokenize(String::from("0e-2"));
        assert_eq!(&result, &[JsonToken::Number(0.0)]);

        let result = tokenize(String::from("1e-2"));
        assert_eq!(&result, &[JsonToken::Number(1e-2)]);

        let result = tokenize(String::from("1.2e+2"));
        assert_eq!(&result, &[JsonToken::Number(1.2e+2)]);

        let result = tokenize(String::from("12e0000"));
        assert_eq!(&result, &[JsonToken::Number(12.0)]);

        let result = tokenize(String::from("1e"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);

        let result = tokenize(String::from("1e+"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);

        let result = tokenize(String::from("1e-"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_big_int() {
        let result = tokenize(String::from("0n"));
        assert_eq!(&result, &[JsonToken::BigInt(BigInt::ZERO)]);

        let result = tokenize(String::from("-0n"));
        assert_eq!(
            &result,
            &[JsonToken::BigInt(BigInt {
                sign: Sign::Negative,
                value: BigUint::ZERO
            })]
        );

        let result = tokenize(String::from("1234567890n"));
        assert_eq!(&result, &[JsonToken::BigInt(BigInt::from_u64(1234567890))]);

        let result = tokenize(String::from("-1234567890n"));
        assert_eq!(&result, &[JsonToken::BigInt(BigInt::from_i64(-1234567890))]);

        let result = tokenize(String::from("123.456n"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Id(String::from("n"))
            ]
        );

        let result = tokenize(String::from("123e456n"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Id(String::from("n"))
            ]
        );

        let result = tokenize(String::from("1234567890na"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Id(String::from("a"))
            ]
        );

        let result = tokenize(String::from("1234567890nn"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Id(String::from("n"))
            ]
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_errors() {
        let result = tokenize(String::from("ᄑ"));
        assert_eq!(
            &result,
            &[JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)]
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_djs() {
        let result = tokenize(String::from("module.exports = "));
        assert_eq!(
            &result,
            &[
                JsonToken::Id(String::from("module")),
                JsonToken::Dot,
                JsonToken::Id(String::from("exports")),
                JsonToken::Equals,
            ]
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_singleline_comments() {
        let result = tokenize(String::from("{//abc\n2\n}"));
        assert_eq!(
            &result,
            &[
                JsonToken::ObjectBegin,
                JsonToken::NewLine,
                JsonToken::Number(2.0),
                JsonToken::NewLine,
                JsonToken::ObjectEnd,
            ]
        );

        let result = tokenize(String::from("0//abc/*"));
        assert_eq!(&result, &[JsonToken::Number(0.0),]);

        let result = tokenize(String::from("0//"));
        assert_eq!(&result, &[JsonToken::Number(0.0),]);

        let result = tokenize(String::from("0/"));
        assert_eq!(
            &result,
            &[
                JsonToken::Number(0.0),
                JsonToken::ErrorToken(ErrorType::UnexpectedCharacter),
            ]
        );

        let result = tokenize(String::from("0/a"));
        assert_eq!(
            &result,
            &[
                JsonToken::Number(0.0),
                JsonToken::ErrorToken(ErrorType::UnexpectedCharacter),
            ]
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_multiline_comments() {
        let result = tokenize(String::from("{/*abc\ndef*/2}"));
        assert_eq!(
            &result,
            &[
                JsonToken::ObjectBegin,
                JsonToken::Number(2.0),
                JsonToken::ObjectEnd,
            ]
        );

        let result = tokenize(String::from("{/*/* /**/2}"));
        assert_eq!(
            &result,
            &[
                JsonToken::ObjectBegin,
                JsonToken::Number(2.0),
                JsonToken::ObjectEnd,
            ]
        );

        let result = tokenize(String::from("{/*"));
        assert_eq!(
            &result,
            &[
                JsonToken::ObjectBegin,
                JsonToken::ErrorToken(ErrorType::CommentClosingExpected),
            ]
        );

        let result = tokenize(String::from("{/**"));
        assert_eq!(
            &result,
            &[
                JsonToken::ObjectBegin,
                JsonToken::ErrorToken(ErrorType::CommentClosingExpected),
            ]
        );
    }
}
