use std::{
    collections::VecDeque,
    mem::take,
    ops::{Deref, RangeInclusive},
};

use crate::{
    big_numbers::{self, big_float::BigFloat, big_int::BigInt, big_uint::BigUint},
    common::{cast::Cast, default::default},
    js::js_bigint::{self, add, equals, from_u64, mul, JsBigintMutRef},
    mem::manager::{Dealloc, Manager},
    range_map::{from_one, from_range, merge, merge_list, RangeMap, State},
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

impl<D: Dealloc> PartialEq for JsonToken<D> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::ErrorToken(l0), Self::ErrorToken(r0)) => l0 == r0,
            (Self::BigInt(l0), Self::BigInt(r0)) => equals(l0, r0),
            (Self::Id(l0), Self::Id(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
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
    ParseZero(js_bigint::Sign),
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

impl<D: Dealloc + 'static> TokenizerState<D> {
    fn push<M: Manager<Dealloc = D> + 'static>(
        self,
        manager: M,
        c: char,
        maps: &TransitionMaps<M>,
    ) -> (Vec<JsonToken<D>>, TokenizerState<D>) {
        match self {
            TokenizerState::Initial => get_next_state(manager, (), c, &maps.initial, maps),
            TokenizerState::ParseId(s) => get_next_state(manager, s, c, &maps.id, maps),
            TokenizerState::ParseString(s) => get_next_state(manager, s, c, &maps.string, maps),
            TokenizerState::ParseEscapeChar(s) => {
                get_next_state(manager, s, c, &maps.escape_char, maps)
            }
            TokenizerState::ParseUnicodeChar(s) => {
                get_next_state(manager, s, c, &maps.unicode_char, maps)
            }
            TokenizerState::ParseZero(s) => get_next_state(manager, s, c, &maps.zero, maps),
            TokenizerState::ParseInt(s) => get_next_state(manager, s, c, &maps.int, maps),
            TokenizerState::ParseMinus => get_next_state(manager, (), c, &maps.minus, maps),
            TokenizerState::ParseFracBegin(s) => {
                get_next_state(manager, s, c, &maps.frac_begin, maps)
            }
            TokenizerState::ParseFrac(s) => get_next_state(manager, s, c, &maps.frac, maps),
            TokenizerState::ParseExpBegin(s) => {
                get_next_state(manager, s, c, &maps.exp_begin, maps)
            }
            TokenizerState::ParseExpSign(s) | TokenizerState::ParseExp(s) => {
                get_next_state(manager, s, c, &maps.exp, maps)
            }
            TokenizerState::ParseBigInt(s) => get_next_state(manager, s, c, &maps.big_int, maps),
            TokenizerState::ParseNewLine => get_next_state(manager, (), c, &maps.new_line, maps),
            TokenizerState::ParseCommentStart => {
                get_next_state(manager, (), c, &maps.comment_start, maps)
            }
            TokenizerState::ParseSinglelineComment => {
                get_next_state(manager, (), c, &maps.singleline_comment, maps)
            }
            TokenizerState::ParseMultilineComment => {
                get_next_state(manager, (), c, &maps.multiline_comment, maps)
            }
            TokenizerState::ParseMultilineCommentAsterix => {
                get_next_state(manager, (), c, &maps.multiline_comment_asterix, maps)
            }
            TokenizerState::ParseOperator(s) => get_next_state(manager, s, c, &maps.operator, maps),
        }
    }

    pub fn push_mut<M: Manager<Dealloc = D> + 'static>(
        &mut self,
        manager: M,
        c: char,
        tm: &TransitionMaps<M>,
    ) -> Vec<JsonToken<M::Dealloc>> {
        let tokens;
        (tokens, *self) = take(self).push(manager, c, tm);
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
    fn push<M: Manager>(
        mut self,
        i: u32,
    ) -> (Vec<JsonToken<M::Dealloc>>, TokenizerState<M::Dealloc>) {
        let new_unicode = self.unicode | (i << ((3 - self.index) * 4));
        match self.index {
            3 => {
                let c = char::from_u32(new_unicode);
                match c {
                    Some(c) => continue_string_state::<M>(self.s, c),
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
    fn from_digit<M: Manager<Dealloc = D>>(
        m: M,
        sign: js_bigint::Sign,
        c: char,
    ) -> JsBigintMutRef<M::Dealloc> {
        from_u64(m, sign, digit_to_number(c))
    }

    fn add_digit<M: Manager<Dealloc = D>>(self, m: M, c: char) -> JsBigintMutRef<M::Dealloc> {
        add(
            m,
            mul(
                m,
                self.deref(),
                from_u64(m, js_bigint::Sign::Positive, 10).deref(),
            )
            .deref(),
            Self::from_digit(m, self.sign(), c).deref(),
        )
    }

    fn into_float_state(self) -> FloatState<D> {
        FloatState { b: self, fe: 0 }
    }

    fn into_exp_state(self) -> ExpState<D> {
        ExpState {
            b: self,
            fe: 0,
            es: js_bigint::Sign::Positive,
            e: 0,
        }
    }

    fn into_old_bigint(self) -> BigInt {
        let deref = self.deref();
        let sign = match deref.sign() {
            js_bigint::Sign::Positive => big_numbers::big_int::Sign::Positive,
            js_bigint::Sign::Negative => big_numbers::big_int::Sign::Negative,
        };
        BigInt {
            sign,
            value: BigUint {
                value: deref.items().to_vec(),
            },
        }
    }

    fn into_token(self) -> JsonToken<D> {
        JsonToken::Number(bigfloat_to_f64(BigFloat {
            significand: self.into_old_bigint(),
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
    fn add_digit<M: Manager<Dealloc = D>>(mut self, m: M, c: char) -> FloatState<M::Dealloc> {
        self.b = self.b.add_digit(m, c);
        self.fe -= 1;
        self
    }

    fn into_exp_state(self) -> ExpState<D> {
        ExpState {
            b: self.b,
            fe: self.fe,
            es: js_bigint::Sign::Positive,
            e: 0,
        }
    }

    fn into_token(self) -> JsonToken<D> {
        JsonToken::Number(bigfloat_to_f64(BigFloat {
            significand: self.b.into_old_bigint(),
            exp: self.fe,
            non_zero_reminder: false,
        }))
    }
}

pub struct ExpState<D: Dealloc> {
    b: JsBigintMutRef<D>,
    fe: i64,
    es: js_bigint::Sign,
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
                js_bigint::Sign::Positive => self.e,
                js_bigint::Sign::Negative => -self.e,
            };
        JsonToken::Number(bigfloat_to_f64(BigFloat {
            significand: self.b.into_old_bigint(),
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

fn start_number<M: Manager>(manager: M, s: js_bigint::Sign, c: char) -> JsBigintMutRef<M::Dealloc> {
    JsBigintMutRef::from_digit(manager, s, c)
}

fn create_range_map<T, M: Manager>(
    list: Vec<RangeInclusive<char>>,
    t: Transition<T, M>,
) -> RangeMap<char, State<Transition<T, M>>> {
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

type Transition<T, M> = TransitionFunc<M, T>;

struct TransitionMap<T, M: Manager> {
    def: Transition<T, M>,
    rm: RangeMap<char, State<Transition<T, M>>>,
}

pub struct TransitionMaps<M: Manager> {
    initial: TransitionMap<(), M>,
    id: TransitionMap<String, M>,
    string: TransitionMap<String, M>,
    escape_char: TransitionMap<String, M>,
    unicode_char: TransitionMap<ParseUnicodeCharState, M>,
    zero: TransitionMap<js_bigint::Sign, M>,
    int: TransitionMap<JsBigintMutRef<M::Dealloc>, M>,
    minus: TransitionMap<(), M>,
    frac_begin: TransitionMap<JsBigintMutRef<M::Dealloc>, M>,
    frac: TransitionMap<FloatState<M::Dealloc>, M>,
    exp_begin: TransitionMap<ExpState<M::Dealloc>, M>,
    exp: TransitionMap<ExpState<M::Dealloc>, M>,
    big_int: TransitionMap<JsBigintMutRef<M::Dealloc>, M>,
    new_line: TransitionMap<(), M>,
    comment_start: TransitionMap<(), M>,
    singleline_comment: TransitionMap<(), M>,
    multiline_comment: TransitionMap<(), M>,
    multiline_comment_asterix: TransitionMap<(), M>,
    operator: TransitionMap<String, M>,
}

pub fn create_transition_maps<M: Manager + 'static>() -> TransitionMaps<M> {
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

fn get_next_state<T: 'static, M: Manager + 'static>(
    manager: M,
    state: T,
    c: char,
    tm: &TransitionMap<T, M>,
    maps: &TransitionMaps<M>,
) -> (Vec<JsonToken<M::Dealloc>>, TokenizerState<M::Dealloc>) {
    let entry = tm.rm.get(c);
    match &entry.value {
        Some(f) => f(manager, state, c, maps),
        None => (tm.def)(manager, state, c, maps),
    }
}

fn create_initial_transitions<M: Manager>() -> TransitionMap<(), M> {
    type Func<M> = TransitionFunc<M, ()>;
    TransitionMap {
        def: (|_, _, _, _| {
            (
                [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].cast(),
                TokenizerState::Initial,
            )
        }) as Func<M>,
        rm: merge_list(
            [
                create_range_map(operator_chars_with_dot(), |_, _, c, _| {
                    (default(), TokenizerState::ParseOperator(c.to_string()))
                }),
                from_range('1'..='9', |manager: M, _, c, _| {
                    (
                        default(),
                        TokenizerState::ParseInt(start_number(
                            manager,
                            js_bigint::Sign::Positive,
                            c,
                        )),
                    )
                }),
                from_one('"', |_, _, _, _| {
                    (default(), TokenizerState::ParseString(String::default()))
                }),
                from_one('0', |_, _, _, _| {
                    (
                        default(),
                        TokenizerState::ParseZero(js_bigint::Sign::Positive),
                    )
                }),
                from_one('-', |_, _, _, _| (default(), TokenizerState::ParseMinus)),
                create_range_map(id_start(), |_, _, c, _| {
                    (default(), TokenizerState::ParseId(c.to_string()))
                }),
                from_one('\n', |_, _, _, _| (default(), TokenizerState::ParseNewLine)),
                create_range_map(set([' ', '\t', '\r']), |_, _, _, _| {
                    (default(), TokenizerState::Initial)
                }),
                from_one('/', |_, _, _, _| {
                    (default(), TokenizerState::ParseCommentStart)
                }),
            ]
            .cast(),
        ),
    }
}

fn create_id_transitions<M: Manager + 'static>() -> TransitionMap<String, M> {
    TransitionMap {
        def: |manager, s, c, maps| {
            transfer_state(
                manager,
                [JsonToken::Id(s)].cast(),
                TokenizerState::Initial,
                c,
                maps,
            )
        },
        rm: create_range_map(id_char(), |_, mut s, c, _| {
            s.push(c);
            (default(), TokenizerState::ParseId(s))
        }),
    }
}

fn create_string_transactions<M: Manager>() -> TransitionMap<String, M> {
    TransitionMap {
        def: |_, mut s, c, _| {
            s.push(c);
            (default(), TokenizerState::ParseString(s))
        },
        rm: merge(
            from_one('"', |_, s, _, _| {
                ([JsonToken::String(s)].cast(), TokenizerState::Initial)
            }),
            from_one('\\', |_, s, _, _| {
                (default(), TokenizerState::ParseEscapeChar(s))
            }),
        ),
    }
}

fn continue_string_state<M: Manager>(
    mut s: String,
    c: char,
) -> (Vec<JsonToken<M::Dealloc>>, TokenizerState<M::Dealloc>) {
    s.push(c);
    (default(), TokenizerState::ParseString(s))
}

fn transfer_state<M: Manager + 'static>(
    manager: M,
    mut vec: Vec<JsonToken<M::Dealloc>>,
    mut state: TokenizerState<M::Dealloc>,
    c: char,
    maps: &TransitionMaps<M>,
) -> (Vec<JsonToken<M::Dealloc>>, TokenizerState<M::Dealloc>) {
    let next_tokens;
    (next_tokens, state) = state.push(manager, c, maps);
    vec.extend(next_tokens);
    (vec, state)
}

fn create_escape_char_transactions<M: Manager + 'static>() -> TransitionMap<String, M> {
    TransitionMap {
        def: |manager, s, c, maps| {
            transfer_state(
                manager,
                [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].cast(),
                TokenizerState::ParseString(s),
                c,
                maps,
            )
        },
        rm: merge_list(
            [
                create_range_map(set(['\"', '\\', '/']), |_, s, c, _| {
                    continue_string_state::<M>(s, c)
                }),
                from_one('b', |_, s, _, _| continue_string_state::<M>(s, '\u{8}')),
                from_one('f', |_, s, _, _| continue_string_state::<M>(s, '\u{c}')),
                from_one('n', |_, s, _, _| continue_string_state::<M>(s, '\n')),
                from_one('r', |_, s, _, _| continue_string_state::<M>(s, '\r')),
                from_one('t', |_, s, _, _| continue_string_state::<M>(s, '\t')),
                from_one('u', |_, s, _, _| {
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

fn create_unicode_char_transactions<M: Manager + 'static>(
) -> TransitionMap<ParseUnicodeCharState, M> {
    type Func<M> = TransitionFunc<M, ParseUnicodeCharState>;
    TransitionMap {
        def: |manager, state, c, maps| {
            transfer_state(
                manager,
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
                    (|_, state, c, _| state.push::<M>(c as u32 - '0' as u32)) as Func<M>,
                ),
                from_range(
                    'a'..='f',
                    (|_, state, c, _| state.push::<M>(c as u32 - ('a' as u32 - 10))) as Func<M>,
                ),
                from_range(
                    'A'..='F',
                    (|_, state, c, _| state.push::<M>(c as u32 - ('A' as u32 - 10))) as Func<M>,
                ),
            ]
            .cast(),
        ),
    }
}

fn create_zero_transactions<M: Manager + 'static>() -> TransitionMap<js_bigint::Sign, M> {
    type Func<M> = TransitionFunc<M, js_bigint::Sign>;
    TransitionMap {
        def: (|manager, _, c, maps| tokenize_invalid_number(manager, c, maps)) as Func<M>,
        rm: merge_list(
            [
                from_one(
                    '.',
                    (|manager, s, _, _| {
                        (
                            default(),
                            TokenizerState::ParseFracBegin(from_u64(manager, s, 0)),
                        )
                    }) as Func<M>,
                ),
                create_range_map(set(['e', 'E']), |manager, s, _, _| {
                    (
                        default(),
                        TokenizerState::ParseExpBegin(ExpState {
                            b: from_u64(manager, s, 0),
                            fe: 0,
                            es: js_bigint::Sign::Positive,
                            e: 0,
                        }),
                    )
                }),
                from_one(
                    'n',
                    (|manager, s, _, _| {
                        (
                            default(),
                            TokenizerState::ParseBigInt(from_u64(manager, s, 0)),
                        )
                    }) as Func<M>,
                ),
                create_range_map(terminal_for_number(), |manager, _, c, maps| {
                    transfer_state(
                        manager,
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

fn create_int_transactions<M: Manager + 'static>() -> TransitionMap<JsBigintMutRef<M::Dealloc>, M> {
    type Func<M> = TransitionFunc<M, JsBigintMutRef<<M as Manager>::Dealloc>>;
    TransitionMap {
        def: (|manager, _, c, maps| tokenize_invalid_number(manager, c, maps)) as Func<M>,
        rm: merge_list(
            [
                from_range(
                    '0'..='9',
                    (|manager, s, c, _| {
                        (default(), TokenizerState::ParseInt(s.add_digit(manager, c)))
                    }) as Func<M>,
                ),
                from_one('.', |_, s, _, _| {
                    (default(), TokenizerState::ParseFracBegin(s))
                }),
                create_range_map(set(['e', 'E']), |_, s, _, _| {
                    (default(), TokenizerState::ParseExpBegin(s.into_exp_state()))
                }),
                from_one('n', |_, s, _, _| {
                    (default(), TokenizerState::ParseBigInt(s))
                }),
                create_range_map(terminal_for_number(), |manager, s, c, maps| {
                    transfer_state(
                        manager,
                        [s.into_token()].cast(),
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

fn create_frac_begin_transactions<M: Manager + 'static>(
) -> TransitionMap<JsBigintMutRef<M::Dealloc>, M> {
    type Func<M> = TransitionFunc<M, JsBigintMutRef<<M as Manager>::Dealloc>>;
    TransitionMap {
        def: (|manager, _, c, maps| tokenize_invalid_number(manager, c, maps)) as Func<M>,
        rm: from_range(
            '0'..='9',
            (|manager, s, c, _| {
                (
                    default(),
                    TokenizerState::ParseFrac(s.into_float_state().add_digit(manager, c)),
                )
            }) as Func<M>,
        ),
    }
}

fn create_frac_transactions<M: Manager + 'static>() -> TransitionMap<FloatState<M::Dealloc>, M> {
    type Func<M> = TransitionFunc<M, FloatState<<M as Manager>::Dealloc>>;
    TransitionMap {
        def: (|manager, _, c, maps| tokenize_invalid_number(manager, c, maps)) as Func<M>,
        rm: merge_list(
            [
                from_range(
                    '0'..='9',
                    (|manager, s, c, _| {
                        (
                            default(),
                            TokenizerState::ParseFrac(s.add_digit(manager, c)),
                        )
                    }) as Func<M>,
                ),
                create_range_map(set(['e', 'E']), |_, s, _, _| {
                    (default(), TokenizerState::ParseExpBegin(s.into_exp_state()))
                }),
                create_range_map(terminal_for_number(), |manager, s, c, maps| {
                    transfer_state(
                        manager,
                        [s.into_token()].cast(),
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

fn create_minus_transactions<M: Manager + 'static>() -> TransitionMap<(), M> {
    type Func<M> = TransitionFunc<M, ()>;
    TransitionMap {
        def: (|manager, _, c, maps| tokenize_invalid_number(manager, c, maps)) as Func<M>,
        rm: merge(
            from_one(
                '0',
                (|_, _, _, _| {
                    (
                        default(),
                        TokenizerState::ParseZero(js_bigint::Sign::Negative),
                    )
                }) as Func<M>,
            ),
            from_range('1'..='9', |manager, _, c, _| {
                (
                    default(),
                    TokenizerState::ParseInt(start_number(manager, js_bigint::Sign::Negative, c)),
                )
            }),
        ),
    }
}

fn create_exp_begin_transactions<M: Manager + 'static>() -> TransitionMap<ExpState<M::Dealloc>, M> {
    type Func<M> = TransitionFunc<M, ExpState<<M as Manager>::Dealloc>>;
    TransitionMap {
        def: (|manager, _, c, maps| tokenize_invalid_number(manager, c, maps)) as Func<M>,
        rm: merge_list(
            [
                from_range(
                    '0'..='9',
                    (|_, s, c, _| (default(), TokenizerState::ParseExp(s.add_digit(c)))) as Func<M>,
                ),
                from_one('+', |_, s, _, _| {
                    (default(), TokenizerState::ParseExpSign(s))
                }),
                from_one('-', |_, mut s, _, _| {
                    (default(), {
                        s.es = js_bigint::Sign::Negative;
                        TokenizerState::ParseExpSign(s)
                    })
                }),
                create_range_map(terminal_for_number(), |manager, s, c, maps| {
                    transfer_state(
                        manager,
                        [s.into_token()].cast(),
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

fn create_exp_transactions<M: Manager + 'static>() -> TransitionMap<ExpState<M::Dealloc>, M> {
    type Func<M> = TransitionFunc<M, ExpState<<M as Manager>::Dealloc>>;
    TransitionMap {
        def: (|manager, _, c, maps| tokenize_invalid_number(manager, c, maps)) as Func<M>,
        rm: merge(
            from_range(
                '0'..='9',
                (|_, s, c, _| (default(), TokenizerState::ParseExp(s.add_digit(c)))) as Func<M>,
            ),
            create_range_map(terminal_for_number(), |manager, s, c, maps| {
                transfer_state(
                    manager,
                    [s.into_token()].cast(),
                    TokenizerState::Initial,
                    c,
                    maps,
                )
            }),
        ),
    }
}

fn create_big_int_transactions<M: Manager + 'static>(
) -> TransitionMap<JsBigintMutRef<M::Dealloc>, M> {
    type Func<M> = TransitionFunc<M, JsBigintMutRef<<M as Manager>::Dealloc>>;
    TransitionMap {
        def: (|manager, _, c, maps| tokenize_invalid_number(manager, c, maps)) as Func<M>,
        rm: create_range_map(terminal_for_number(), |manager, s, c, maps| {
            transfer_state(
                manager,
                [s.into_token()].cast(),
                TokenizerState::Initial,
                c,
                maps,
            )
        }),
    }
}

fn tokenize_invalid_number<M: Manager + 'static>(
    manager: M,
    c: char,
    maps: &TransitionMaps<M>,
) -> (Vec<JsonToken<M::Dealloc>>, TokenizerState<M::Dealloc>) {
    transfer_state(
        manager,
        [JsonToken::ErrorToken(ErrorType::InvalidNumber)].cast(),
        TokenizerState::Initial,
        c,
        maps,
    )
}

fn create_new_line_transactions<M: Manager + 'static>() -> TransitionMap<(), M> {
    type Func<M> = TransitionFunc<M, ()>;
    TransitionMap {
        def: (|manager, _, c, maps| {
            transfer_state(
                manager,
                [JsonToken::NewLine].cast(),
                TokenizerState::Initial,
                c,
                maps,
            )
        }) as Func<M>,
        rm: create_range_map(set(WHITE_SPACE_CHARS), |_, _, _, _| {
            (default(), TokenizerState::ParseNewLine)
        }),
    }
}

fn create_comment_start_transactions<M: Manager>() -> TransitionMap<(), M> {
    type Func<M> = TransitionFunc<M, ()>;
    TransitionMap {
        def: (|_, _, _, _| {
            (
                [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].cast(),
                TokenizerState::Initial,
            )
        }) as Func<M>,
        rm: merge(
            from_one('/', |_, _, _, _| {
                (default(), TokenizerState::ParseSinglelineComment)
            }),
            from_one('*', |_, _, _, _| {
                (default(), TokenizerState::ParseMultilineComment)
            }),
        ),
    }
}

type TransitionFunc<M, S> = fn(
    m: M,
    s: S,
    c: char,
    maps: &TransitionMaps<M>,
) -> (
    Vec<JsonToken<<M as Manager>::Dealloc>>,
    TokenizerState<<M as Manager>::Dealloc>,
);

fn create_singleline_comment_transactions<M: Manager>() -> TransitionMap<(), M> {
    type Func<M> = TransitionFunc<M, ()>;
    TransitionMap {
        def: (|_, _, _, _| (default(), TokenizerState::ParseSinglelineComment)) as Func<M>,
        rm: create_range_map(set(WHITE_SPACE_CHARS), |_, _, _, _| {
            (default(), TokenizerState::ParseNewLine)
        }),
    }
}

fn create_multiline_comment_transactions<M: Manager>() -> TransitionMap<(), M> {
    type Func<M> = fn(
        m: M,
        s: (),
        c: char,
        maps: &TransitionMaps<M>,
    ) -> (
        Vec<JsonToken<<M as Manager>::Dealloc>>,
        TokenizerState<<M as Manager>::Dealloc>,
    );
    TransitionMap {
        def: (|_, _, _, _| (default(), TokenizerState::ParseMultilineComment)) as Func<M>,
        rm: from_one('*', |_, _, _, _| {
            (default(), TokenizerState::ParseMultilineCommentAsterix)
        }),
    }
}

fn create_multiline_comment_asterix_transactions<M: Manager>() -> TransitionMap<(), M> {
    type Func<M> = fn(
        m: M,
        s: (),
        c: char,
        maps: &TransitionMaps<M>,
    ) -> (
        Vec<JsonToken<<M as Manager>::Dealloc>>,
        TokenizerState<<M as Manager>::Dealloc>,
    );
    TransitionMap {
        def: (|_, _, _, _| (default(), TokenizerState::ParseMultilineComment)) as Func<M>,
        rm: merge(
            from_one('/', |_, _, _, _| (default(), TokenizerState::Initial)),
            from_one('*', |_, _, _, _| {
                (default(), TokenizerState::ParseMultilineCommentAsterix)
            }),
        ),
    }
}

fn create_operator_transactions<M: Manager + 'static>() -> TransitionMap<String, M> {
    TransitionMap {
        def: |manager, s, c, maps| {
            let token = operator_to_token(s).unwrap();
            transfer_state(manager, [token].cast(), TokenizerState::Initial, c, maps)
        },
        rm: create_range_map(operator_chars_with_dot(), |manager, s, c, maps| {
            let mut next_string = s.clone();
            next_string.push(c);
            match operator_to_token::<M::Dealloc>(next_string) {
                Some(_) => {
                    let mut next_string = s.clone();
                    next_string.push(c);
                    (default(), TokenizerState::ParseOperator(next_string))
                }
                _ => {
                    let token = operator_to_token(s).unwrap();
                    transfer_state(manager, [token].cast(), TokenizerState::Initial, c, maps)
                }
            }
        }),
    }
}

pub fn tokenize<M: Manager + 'static>(manager: M, input: String) -> Vec<JsonToken<M::Dealloc>> {
    TokenizerStateIterator::new(manager, input.chars()).collect()
}

pub struct TokenizerStateIterator<T: Iterator<Item = char>, M: Manager> {
    manager: M,
    chars: T,
    cache: VecDeque<JsonToken<M::Dealloc>>,
    state: TokenizerState<M::Dealloc>,
    maps: TransitionMaps<M>,
    end: bool,
}

impl<T: Iterator<Item = char>, M: Manager + 'static> TokenizerStateIterator<T, M> {
    pub fn new(manager: M, chars: T) -> Self {
        Self {
            manager,
            chars,
            cache: default(),
            state: default(),
            maps: create_transition_maps(),
            end: false,
        }
    }
}

impl<T: Iterator<Item = char>, M: Manager + 'static> Iterator for TokenizerStateIterator<T, M> {
    type Item = JsonToken<M::Dealloc>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(result) = self.cache.pop_front() {
                return Some(result);
            }
            if self.end {
                return None;
            }
            match self.chars.next() {
                Some(c) => self
                    .cache
                    .extend(self.state.push_mut(self.manager, c, &self.maps)),
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
        js::js_bigint::{self, from_u64, zero},
        mem::global::{Global, GLOBAL},
        tokenizer::bigfloat_to_f64,
    };

    use super::{tokenize, ErrorType, JsonToken};

    #[test]
    #[wasm_bindgen_test]
    fn test_empty() {
        let result = tokenize(GLOBAL, String::from(""));
        assert_eq!(result.len(), 0);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_ops() {
        let result = tokenize(GLOBAL, String::from("{"));
        assert_eq!(result[0], JsonToken::<Global>::ObjectBegin);

        let result = tokenize(GLOBAL, String::from("}"));
        assert_eq!(&result, &[JsonToken::ObjectEnd]);

        let result = tokenize(GLOBAL, String::from("["));
        assert_eq!(&result, &[JsonToken::ArrayBegin]);

        let result = tokenize(GLOBAL, String::from("]"));
        assert_eq!(&result, &[JsonToken::ArrayEnd]);

        let result = tokenize(GLOBAL, String::from(":"));
        assert_eq!(&result, &[JsonToken::Colon]);

        let result = tokenize(GLOBAL, String::from(","));
        assert_eq!(&result, &[JsonToken::Comma]);

        let result = tokenize(GLOBAL, String::from("="));
        assert_eq!(&result, &[JsonToken::Equals]);

        let result = tokenize(GLOBAL, String::from("."));
        assert_eq!(&result, &[JsonToken::Dot]);

        let result = tokenize(GLOBAL, String::from(";"));
        assert_eq!(&result, &[JsonToken::Semicolon]);

        let result = tokenize(GLOBAL, String::from("()"));
        assert_eq!(
            &result,
            &[JsonToken::OpeningParenthesis, JsonToken::ClosingParenthesis]
        );

        let result = tokenize(GLOBAL, String::from("[{ :, }]"));
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
        let result = tokenize(GLOBAL, String::from("true"));
        assert_eq!(&result, &[JsonToken::Id(String::from("true"))]);

        let result = tokenize(GLOBAL, String::from("false"));
        assert_eq!(&result, &[JsonToken::Id(String::from("false"))]);

        let result = tokenize(GLOBAL, String::from("null"));
        assert_eq!(&result, &[JsonToken::Id(String::from("null"))]);

        let result = tokenize(GLOBAL, String::from("tru tru"));
        assert_eq!(
            &result,
            &[
                JsonToken::Id(String::from("tru")),
                JsonToken::Id(String::from("tru")),
            ]
        );

        let result = tokenize(GLOBAL, String::from("ABCxyz_0123456789$"));
        assert_eq!(
            &result,
            &[JsonToken::Id(String::from("ABCxyz_0123456789$")),]
        );

        let result = tokenize(GLOBAL, String::from("_"));
        assert_eq!(&result, &[JsonToken::Id(String::from("_")),]);

        let result = tokenize(GLOBAL, String::from("$"));
        assert_eq!(&result, &[JsonToken::Id(String::from("$")),]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_whitespace() {
        let result = tokenize(GLOBAL, String::from(" \t\n\r"));
        assert_eq!(&result, &[]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_string() {
        let result = tokenize(GLOBAL, String::from("\"\""));
        assert_eq!(&result, &[JsonToken::String("".to_string())]);

        let result = tokenize(GLOBAL, String::from("\"value\""));
        assert_eq!(&result, &[JsonToken::String("value".to_string())]);

        let result = tokenize(GLOBAL, String::from("\"value1\" \"value2\""));
        assert_eq!(
            &result,
            &[
                JsonToken::String("value1".to_string()),
                JsonToken::String("value2".to_string())
            ]
        );

        let result = tokenize(GLOBAL, String::from("\"value"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::MissingQuotes)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_escaped_characters() {
        let result = tokenize(GLOBAL, String::from("\"\\b\\f\\n\\r\\t\""));
        assert_eq!(
            &result,
            &[JsonToken::String("\u{8}\u{c}\n\r\t".to_string())]
        );

        let result = tokenize(GLOBAL, String::from("\"\\x\""));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::UnexpectedCharacter),
                JsonToken::String("x".to_string())
            ]
        );

        let result = tokenize(GLOBAL, String::from("\"\\"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::MissingQuotes)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_unicode() {
        let result = tokenize(GLOBAL, String::from("\"\\u1234\""));
        assert_eq!(&result, &[JsonToken::String("ሴ".to_string())]);

        let result = tokenize(GLOBAL, String::from("\"\\uaBcDEeFf\""));
        assert_eq!(&result, &[JsonToken::String("ꯍEeFf".to_string())]);

        let result = tokenize(GLOBAL, String::from("\"\\uEeFg\""));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidHex),
                JsonToken::String("g".to_string())
            ]
        );

        let result = tokenize(GLOBAL, String::from("\"\\uEeF"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::MissingQuotes)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_integer() {
        let result = tokenize(GLOBAL, String::from("0"));
        assert_eq!(&result, &[JsonToken::Number(0.0)]);

        let result = tokenize(GLOBAL, String::from("-0"));
        assert_eq!(&result, &[JsonToken::Number(0.0)]);

        let result = tokenize(GLOBAL, String::from("0abc"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Id(String::from("abc"))
            ]
        );

        let result = tokenize(GLOBAL, String::from("0. 2"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Number(2.0)
            ]
        );

        let result = tokenize(GLOBAL, String::from("1234567890"));
        assert_eq!(&result, &[JsonToken::Number(1234567890.0)]);

        let result = tokenize(GLOBAL, String::from("-1234567890"));
        assert_eq!(&result, &[JsonToken::Number(-1234567890.0)]);

        let result = tokenize(GLOBAL, String::from("[0,1]"));
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

        let result = tokenize(GLOBAL, String::from("001"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Number(1.0),
            ]
        );

        let result = tokenize(GLOBAL, String::from("-"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);

        let result = tokenize(GLOBAL, String::from("-{}"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::ObjectBegin,
                JsonToken::ObjectEnd
            ]
        );

        let result = tokenize(GLOBAL, String::from("9007199254740991"));
        assert_eq!(&result, &[JsonToken::Number(9007199254740991.0)]);

        let result = tokenize(GLOBAL, String::from("9007199254740992"));
        assert_eq!(&result, &[JsonToken::Number(9007199254740992.0)]);

        let result = tokenize(GLOBAL, String::from("9007199254740993"));
        assert_eq!(&result, &[JsonToken::Number(9007199254740993.0)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_big_float() {
        let result = tokenize(
            GLOBAL,
            String::from("340282366920938463463374607431768211456"),
        );
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
        let result = tokenize(GLOBAL, String::from("0.01"));
        assert_eq!(&result, &[JsonToken::Number(0.01)]);

        let result = tokenize(GLOBAL, String::from("[-12.34]"));
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
        let result = tokenize(GLOBAL, String::from("1e1000"));
        assert_eq!(&result, &[JsonToken::Number(f64::INFINITY)]);

        let result = tokenize(GLOBAL, String::from("-1e+1000"));
        assert_eq!(&result, &[JsonToken::Number(f64::NEG_INFINITY)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_exp() {
        let result = tokenize(GLOBAL, String::from("1e2"));
        assert_eq!(&result, &[JsonToken::Number(1e2)]);

        let result = tokenize(GLOBAL, String::from("1E+2"));
        assert_eq!(&result, &[JsonToken::Number(1e2)]);

        let result = tokenize(GLOBAL, String::from("0e-2"));
        assert_eq!(&result, &[JsonToken::Number(0.0)]);

        let result = tokenize(GLOBAL, String::from("1e-2"));
        assert_eq!(&result, &[JsonToken::Number(1e-2)]);

        let result = tokenize(GLOBAL, String::from("1.2e+2"));
        assert_eq!(&result, &[JsonToken::Number(1.2e+2)]);

        let result = tokenize(GLOBAL, String::from("12e0000"));
        assert_eq!(&result, &[JsonToken::Number(12.0)]);

        let result = tokenize(GLOBAL, String::from("1e"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);

        let result = tokenize(GLOBAL, String::from("1e+"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);

        let result = tokenize(GLOBAL, String::from("1e-"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_big_int() {
        let result = tokenize(GLOBAL, String::from("0n"));
        assert_eq!(&result, &[JsonToken::BigInt(zero(GLOBAL))]);

        let result = tokenize(GLOBAL, String::from("-0n"));
        assert_eq!(
            &result,
            &[JsonToken::BigInt(from_u64(
                GLOBAL,
                js_bigint::Sign::Negative,
                0
            ))]
        );

        let result = tokenize(GLOBAL, String::from("1234567890n"));
        assert_eq!(
            &result,
            &[JsonToken::BigInt(from_u64(
                GLOBAL,
                js_bigint::Sign::Positive,
                1234567890
            ))]
        );

        let result = tokenize(GLOBAL, String::from("-1234567890n"));
        assert_eq!(
            &result,
            &[JsonToken::BigInt(from_u64(
                GLOBAL,
                js_bigint::Sign::Negative,
                1234567890
            ))]
        );

        let result = tokenize(GLOBAL, String::from("123.456n"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Id(String::from("n"))
            ]
        );

        let result = tokenize(GLOBAL, String::from("123e456n"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Id(String::from("n"))
            ]
        );

        let result = tokenize(GLOBAL, String::from("1234567890na"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Id(String::from("a"))
            ]
        );

        let result = tokenize(GLOBAL, String::from("1234567890nn"));
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
        let result = tokenize(GLOBAL, String::from("ᄑ"));
        assert_eq!(
            &result,
            &[JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)]
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_djs() {
        let result = tokenize(GLOBAL, String::from("module.exports = "));
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
        let result = tokenize(GLOBAL, String::from("{//abc\n2\n}"));
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

        let result = tokenize(GLOBAL, String::from("0//abc/*"));
        assert_eq!(&result, &[JsonToken::Number(0.0),]);

        let result = tokenize(GLOBAL, String::from("0//"));
        assert_eq!(&result, &[JsonToken::Number(0.0),]);

        let result = tokenize(GLOBAL, String::from("0/"));
        assert_eq!(
            &result,
            &[
                JsonToken::Number(0.0),
                JsonToken::ErrorToken(ErrorType::UnexpectedCharacter),
            ]
        );

        let result = tokenize(GLOBAL, String::from("0/a"));
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
        let result = tokenize(GLOBAL, String::from("{/*abc\ndef*/2}"));
        assert_eq!(
            &result,
            &[
                JsonToken::ObjectBegin,
                JsonToken::Number(2.0),
                JsonToken::ObjectEnd,
            ]
        );

        let result = tokenize(GLOBAL, String::from("{/*/* /**/2}"));
        assert_eq!(
            &result,
            &[
                JsonToken::ObjectBegin,
                JsonToken::Number(2.0),
                JsonToken::ObjectEnd,
            ]
        );

        let result = tokenize(GLOBAL, String::from("{/*"));
        assert_eq!(
            &result,
            &[
                JsonToken::ObjectBegin,
                JsonToken::ErrorToken(ErrorType::CommentClosingExpected),
            ]
        );

        let result = tokenize(GLOBAL, String::from("{/**"));
        assert_eq!(
            &result,
            &[
                JsonToken::ObjectBegin,
                JsonToken::ErrorToken(ErrorType::CommentClosingExpected),
            ]
        );
    }
}
