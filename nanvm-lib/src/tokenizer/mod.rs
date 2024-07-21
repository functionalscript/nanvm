use std::{collections::VecDeque, mem::take, ops::RangeInclusive};

use crate::{
    big_numbers::{
        big_float::BigFloat,
        big_int::{BigInt, Sign},
        big_uint::BigUint,
    },
    common::{cast::Cast, default::default},
    range_map::{from_one, from_range, merge, merge_list, RangeMap, State},
};

#[derive(Debug, PartialEq)]
pub enum JsonToken {
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
    BigInt(BigInt),
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
enum TokenizerState {
    #[default]
    Initial,
    ParseId(String),
    ParseString(String),
    ParseEscapeChar(String),
    ParseUnicodeChar(ParseUnicodeCharState),
    ParseMinus,
    ParseZero(Sign),
    ParseInt(IntegerState),
    ParseFracBegin(IntegerState),
    ParseFrac(FloatState),
    ParseExpBegin(ExpState),
    ParseExpSign(ExpState),
    ParseExp(ExpState),
    ParseBigInt(IntegerState),
    ParseNewLine,
    ParseCommentStart,
    ParseSinglelineComment,
    ParseMultilineComment,
    ParseMultilineCommentAsterix,
    ParseOperator(String),
}

impl TokenizerState {
    fn push(self, c: char) -> (Vec<JsonToken>, TokenizerState) {
        match self {
            TokenizerState::Initial => tokenize_initial(c),
            TokenizerState::ParseId(s) => tokenize_id(s, c),
            TokenizerState::ParseString(s) => tokenize_string(s, c),
            TokenizerState::ParseEscapeChar(s) => tokenize_escape_char(s, c),
            TokenizerState::ParseUnicodeChar(s) => tokenize_unicode_char(s, c),
            TokenizerState::ParseZero(s) => tokenize_zero(s, c),
            TokenizerState::ParseInt(s) => tokenize_integer(s, c),
            TokenizerState::ParseMinus => tokenize_minus(c),
            TokenizerState::ParseFracBegin(s) => tokenize_frac_begin(s, c),
            TokenizerState::ParseFrac(s) => tokenize_frac(s, c),
            TokenizerState::ParseExpBegin(s) => tokenize_exp_begin(s, c),
            TokenizerState::ParseExpSign(s) | TokenizerState::ParseExp(s) => tokenize_exp(s, c),
            TokenizerState::ParseBigInt(s) => tokenize_big_int(s, c),
            TokenizerState::ParseNewLine => tokenize_new_line(c),
            TokenizerState::ParseCommentStart => tokenize_comment_start(c),
            TokenizerState::ParseSinglelineComment => tokenize_singleline_comment(c),
            TokenizerState::ParseMultilineComment => tokenize_multiline_comment(c),
            TokenizerState::ParseMultilineCommentAsterix => tokenize_multiline_comment_asterix(c),
            TokenizerState::ParseOperator(s) => tokenize_operator(s, c),
        }
    }

    fn push_mut(&mut self, c: char) -> Vec<JsonToken> {
        let tokens;
        (tokens, *self) = take(self).push(c);
        tokens
    }

    fn end(self) -> Vec<JsonToken> {
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

struct ParseUnicodeCharState {
    s: String,
    unicode: u32,
    index: u8,
}

impl ParseUnicodeCharState {
    fn push(mut self, i: u32) -> (Vec<JsonToken>, TokenizerState) {
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

struct IntegerState {
    s: Sign,
    b: BigUint,
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

impl IntegerState {
    fn from_difit(sign: Sign, c: char) -> IntegerState {
        IntegerState {
            s: sign,
            b: BigUint::from_u64(digit_to_number(c)),
        }
    }

    fn add_digit(mut self, c: char) -> IntegerState {
        self.b = self.b.add_digit(c);
        self
    }

    fn into_float_state(self) -> FloatState {
        FloatState {
            s: self.s,
            b: self.b,
            fe: 0,
        }
    }

    fn into_exp_state(self) -> ExpState {
        ExpState {
            s: self.s,
            b: self.b,
            fe: 0,
            es: Sign::Positive,
            e: 0,
        }
    }

    fn into_token(self) -> JsonToken {
        JsonToken::Number(bigfloat_to_f64(BigFloat {
            significand: BigInt {
                sign: self.s,
                value: self.b,
            },
            exp: 0,
            non_zero_reminder: false,
        }))
    }

    fn into_big_int_token(self) -> JsonToken {
        JsonToken::BigInt(BigInt {
            sign: self.s,
            value: self.b,
        })
    }
}

struct FloatState {
    s: Sign,
    b: BigUint,
    fe: i64,
}

impl FloatState {
    fn add_digit(mut self, c: char) -> FloatState {
        self.b = self.b.add_digit(c);
        self.fe -= 1;
        self
    }

    fn into_exp_state(self) -> ExpState {
        ExpState {
            s: self.s,
            b: self.b,
            fe: self.fe,
            es: Sign::Positive,
            e: 0,
        }
    }

    fn into_token(self) -> JsonToken {
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

struct ExpState {
    s: Sign,
    b: BigUint,
    fe: i64,
    es: Sign,
    e: i64,
}

impl ExpState {
    const fn add_digit(mut self, c: char) -> ExpState {
        self.e = self.e * 10 + digit_to_number(c) as i64;
        self
    }

    fn into_token(self) -> JsonToken {
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

const fn is_new_line(c: char) -> bool {
    matches!(c, '\n')
}

const fn is_white_space(c: char) -> bool {
    matches!(c, ' ' | '\n' | '\t' | '\r')
}

fn is_operator(c: char) -> bool {
    matches!(
        c,
        '{' | '}' | '[' | ']' | ':' | ',' | '=' | '.' | ';' | '(' | ')'
    )
}

fn operator_to_token(s: String) -> Option<JsonToken> {
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

const fn is_id_start(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '$')
}

const fn is_id_char(c: char) -> bool {
    match c {
        '0'..='9' => true,
        c if is_id_start(c) => true,
        _ => false,
    }
}

const WHITE_SPACE_CHARS: [char; 4] = [' ', '\n', '\t', '\r'];
const OPERATOR_CHARS: [char; 10] = ['{', '}', '[', ']', ':', ',', '=', ';', '(', ')'];

fn id_start() -> Vec<RangeInclusive<char>> {
    ['a'..='z', 'A'..='Z', one('_'), one('$')].cast()
}

fn id() -> Vec<RangeInclusive<char>> {
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

fn is_terminal_for_number(c: char) -> bool {
    match c {
        '"' | '/' => true,
        c if is_white_space(c) => true,
        c if is_operator(c) => true,
        _ => false,
    }
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

type Transition<T> = fn(state: T, c: char) -> (Vec<JsonToken>, TokenizerState);

fn get_next_state<T>(
    state: T,
    c: char,
    def: Transition<T>,
    rm: RangeMap<char, State<Transition<T>>>,
) -> (Vec<JsonToken>, TokenizerState)
where
    T: 'static,
{
    let entry = rm.get(c);
    match &entry.value {
        Some(f) => f(state, c),
        None => def(state, c),
    }
}

fn tokenize_initial(c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: (), c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        (),
        c,
        (|_, _| {
            (
                [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].cast(),
                TokenizerState::Initial,
            )
        }) as Func,
        merge_list(
            [
                create_range_map(operator_chars_with_dot(), |_, c| {
                    (default(), TokenizerState::ParseOperator(c.to_string()))
                }),
                from_range('1'..='9', |_, c| {
                    (
                        default(),
                        TokenizerState::ParseInt(start_number(Sign::Positive, c)),
                    )
                }),
                from_one('"', |_, _| {
                    (default(), TokenizerState::ParseString(String::default()))
                }),
                from_one('0', |_, _| {
                    (default(), TokenizerState::ParseZero(Sign::Positive))
                }),
                from_one('-', |_, _| (default(), TokenizerState::ParseMinus)),
                create_range_map(id_start(), |_, c| {
                    (default(), TokenizerState::ParseId(c.to_string()))
                }),
                from_one('\n', |_, _| (default(), TokenizerState::ParseNewLine)),
                create_range_map(set([' ', '\t', '\r']), |_, _| {
                    (default(), TokenizerState::Initial)
                }),
                from_one('/', |_, _| (default(), TokenizerState::ParseCommentStart)),
            ]
            .cast(),
        ),
    )
}

fn tokenize_id(s: String, c: char) -> (Vec<JsonToken>, TokenizerState) {
    get_next_state(
        s,
        c,
        |s, c| transfer_state([JsonToken::Id(s)].cast(), TokenizerState::Initial, c),
        create_range_map(id(), |mut s, c| {
            s.push(c);
            (default(), TokenizerState::ParseId(s))
        }),
    )
}

fn tokenize_string(s: String, c: char) -> (Vec<JsonToken>, TokenizerState) {
    get_next_state(
        s,
        c,
        |mut s, c| {
            s.push(c);
            (default(), TokenizerState::ParseString(s))
        },
        merge(
            from_one('"', |s, _| {
                ([JsonToken::String(s)].cast(), TokenizerState::Initial)
            }),
            from_one('\\', |s, _| (default(), TokenizerState::ParseEscapeChar(s))),
        ),
    )
}

fn continue_string_state(mut s: String, c: char) -> (Vec<JsonToken>, TokenizerState) {
    s.push(c);
    (default(), TokenizerState::ParseString(s))
}

fn transfer_state(
    mut vec: Vec<JsonToken>,
    mut state: TokenizerState,
    c: char,
) -> (Vec<JsonToken>, TokenizerState) {
    let next_tokens;
    (next_tokens, state) = state.push(c);
    vec.extend(next_tokens);
    (vec, state)
}

fn tokenize_escape_char(s: String, c: char) -> (Vec<JsonToken>, TokenizerState) {
    get_next_state(
        s,
        c,
        |s, c| {
            transfer_state(
                [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].cast(),
                TokenizerState::ParseString(s),
                c,
            )
        },
        merge_list(
            [
                create_range_map(set(['\"', '\\', '/']), continue_string_state),
                from_one('b', |s, _| continue_string_state(s, '\u{8}')),
                from_one('f', |s, _| continue_string_state(s, '\u{c}')),
                from_one('n', |s, _| continue_string_state(s, '\n')),
                from_one('r', |s, _| continue_string_state(s, '\r')),
                from_one('t', |s, _| continue_string_state(s, '\t')),
                from_one('u', |s, _| {
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
    )
}

fn tokenize_unicode_char(
    state: ParseUnicodeCharState,
    c: char,
) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(state: ParseUnicodeCharState, c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        state,
        c,
        |state, c| {
            transfer_state(
                [JsonToken::ErrorToken(ErrorType::InvalidHex)].cast(),
                TokenizerState::ParseString(state.s),
                c,
            )
        },
        merge_list(
            [
                from_range(
                    '0'..='9',
                    (|state, c| state.push(c as u32 - '0' as u32)) as Func,
                ),
                from_range(
                    'a'..='f',
                    (|state, c| state.push(c as u32 - ('a' as u32 - 10))) as Func,
                ),
                from_range(
                    'A'..='F',
                    (|state, c| state.push(c as u32 - ('A' as u32 - 10))) as Func,
                ),
            ]
            .cast(),
        ),
    )
}

fn tokenize_zero(s: Sign, c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: Sign, c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        s,
        c,
        (|_, c| tokenize_invalid_number(c)) as Func,
        merge_list(
            [
                from_one(
                    '.',
                    (|s, _| {
                        (
                            default(),
                            TokenizerState::ParseFracBegin(IntegerState {
                                s,
                                b: BigUint::ZERO,
                            }),
                        )
                    }) as Func,
                ),
                create_range_map(set(['e', 'E']), |s, _| {
                    (
                        default(),
                        TokenizerState::ParseExpBegin(ExpState {
                            s,
                            b: BigUint::ZERO,
                            fe: 0,
                            es: Sign::Positive,
                            e: 0,
                        }),
                    )
                }),
                from_one('n', |s, _| {
                    (
                        default(),
                        TokenizerState::ParseBigInt(IntegerState {
                            s,
                            b: BigUint::ZERO,
                        }),
                    )
                }),
                create_range_map(terminal_for_number(), |_, c| {
                    transfer_state(
                        [JsonToken::Number(default())].cast(),
                        TokenizerState::Initial,
                        c,
                    )
                }),
            ]
            .cast(),
        ),
    )
}

fn tokenize_integer(s: IntegerState, c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: IntegerState, c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        s,
        c,
        (|_, c| tokenize_invalid_number(c)) as Func,
        merge_list(
            [
                from_range(
                    '0'..='9',
                    (|s, c| (default(), TokenizerState::ParseInt(s.add_digit(c)))) as Func,
                ),
                from_one('.', |s, _| (default(), TokenizerState::ParseFracBegin(s))),
                create_range_map(set(['e', 'E']), |s, _| {
                    (default(), TokenizerState::ParseExpBegin(s.into_exp_state()))
                }),
                from_one('n', |s, _| (default(), TokenizerState::ParseBigInt(s))),
                create_range_map(terminal_for_number(), |s, c| {
                    transfer_state([s.into_token()].cast(), TokenizerState::Initial, c)
                }),
            ]
            .cast(),
        ),
    )
}

fn tokenize_frac_begin(s: IntegerState, c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: IntegerState, c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        s,
        c,
        (|_, c| tokenize_invalid_number(c)) as Func,
        from_range(
            '0'..='9',
            (|s, c| {
                (
                    default(),
                    TokenizerState::ParseFrac(s.into_float_state().add_digit(c)),
                )
            }) as Func,
        ),
    )
}

fn tokenize_frac(s: FloatState, c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: FloatState, c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        s,
        c,
        (|_, c| tokenize_invalid_number(c)) as Func,
        merge_list(
            [
                from_range(
                    '0'..='9',
                    (|s, c| (default(), TokenizerState::ParseFrac(s.add_digit(c)))) as Func,
                ),
                create_range_map(set(['e', 'E']), |s, _| {
                    (default(), TokenizerState::ParseExpBegin(s.into_exp_state()))
                }),
                create_range_map(terminal_for_number(), |s, c| {
                    transfer_state([s.into_token()].cast(), TokenizerState::Initial, c)
                }),
            ]
            .cast(),
        ),
    )
}

fn tokenize_minus(c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: (), c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        (),
        c,
        (|_, c| tokenize_invalid_number(c)) as Func,
        merge(
            from_one(
                '0',
                (|_, _| (default(), TokenizerState::ParseZero(Sign::Negative))) as Func,
            ),
            from_range('1'..='9', |_, c| {
                (
                    default(),
                    TokenizerState::ParseInt(start_number(Sign::Negative, c)),
                )
            }),
        ),
    )
}

fn tokenize_exp_begin(s: ExpState, c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: ExpState, c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        s,
        c,
        (|_, c| tokenize_invalid_number(c)) as Func,
        merge_list(
            [
                from_range(
                    '0'..='9',
                    (|s, c| (default(), TokenizerState::ParseExp(s.add_digit(c)))) as Func,
                ),
                from_one('+', |s, _| (default(), TokenizerState::ParseExpSign(s))),
                from_one('-', |mut s, _| {
                    (default(), {
                        s.es = Sign::Negative;
                        TokenizerState::ParseExpSign(s)
                    })
                }),
                create_range_map(terminal_for_number(), |s, c| {
                    transfer_state([s.into_token()].cast(), TokenizerState::Initial, c)
                }),
            ]
            .cast(),
        ),
    )
}

fn tokenize_exp(s: ExpState, c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: ExpState, c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        s,
        c,
        (|_, c| tokenize_invalid_number(c)) as Func,
        merge(
            from_range(
                '0'..='9',
                (|s, c| (default(), TokenizerState::ParseExp(s.add_digit(c)))) as Func,
            ),
            create_range_map(terminal_for_number(), |s, c| {
                transfer_state([s.into_token()].cast(), TokenizerState::Initial, c)
            }),
        ),
    )
}

fn tokenize_big_int(s: IntegerState, c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: IntegerState, c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        s,
        c,
        (|_, c| tokenize_invalid_number(c)) as Func,
        create_range_map(terminal_for_number(), |s, c| {
            transfer_state([s.into_token()].cast(), TokenizerState::Initial, c)
        }),
    )
}

fn tokenize_invalid_number(c: char) -> (Vec<JsonToken>, TokenizerState) {
    transfer_state(
        [JsonToken::ErrorToken(ErrorType::InvalidNumber)].cast(),
        TokenizerState::Initial,
        c,
    )
}

fn tokenize_new_line(c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: (), c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        (),
        c,
        (|_, c| transfer_state([JsonToken::NewLine].cast(), TokenizerState::Initial, c)) as Func,
        create_range_map(set(WHITE_SPACE_CHARS), |_, _| {
            (default(), TokenizerState::ParseNewLine)
        }),
    )
}

fn tokenize_comment_start(c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: (), c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        (),
        c,
        (|_, _| {
            (
                [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].cast(),
                TokenizerState::Initial,
            )
        }) as Func,
        merge(
            from_one('/', |_, _| {
                (default(), TokenizerState::ParseSinglelineComment)
            }),
            from_one('*', |_, _| {
                (default(), TokenizerState::ParseMultilineComment)
            }),
        ),
    )
}

fn tokenize_singleline_comment(c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: (), c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        (),
        c,
        (|_, _| (default(), TokenizerState::ParseSinglelineComment)) as Func,
        create_range_map(set(WHITE_SPACE_CHARS), |_, _| {
            (default(), TokenizerState::ParseNewLine)
        }),
    )
}

fn tokenize_multiline_comment(c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: (), c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        (),
        c,
        (|_, _| (default(), TokenizerState::ParseMultilineComment)) as Func,
        from_one('*', |_, _| {
            (default(), TokenizerState::ParseMultilineCommentAsterix)
        }),
    )
}

fn tokenize_multiline_comment_asterix(c: char) -> (Vec<JsonToken>, TokenizerState) {
    type Func = fn(s: (), c: char) -> (Vec<JsonToken>, TokenizerState);
    get_next_state(
        (),
        c,
        (|_, _| (default(), TokenizerState::ParseMultilineComment)) as Func,
        merge(
            from_one('/', |_, _| (default(), TokenizerState::Initial)),
            from_one('*', |_, _| {
                (default(), TokenizerState::ParseMultilineCommentAsterix)
            }),
        ),
    )
}

fn tokenize_operator(s: String, c: char) -> (Vec<JsonToken>, TokenizerState) {
    get_next_state(
        s,
        c,
        |s, c| {
            let token = operator_to_token(s).unwrap();
            transfer_state([token].cast(), TokenizerState::Initial, c)
        },
        create_range_map(operator_chars_with_dot(), |s, c| {
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
                    transfer_state([token].cast(), TokenizerState::Initial, c)
                }
            }
        }),
    )
}

pub fn tokenize(input: String) -> Vec<JsonToken> {
    TokenizerStateIterator::new(input.chars()).collect()
}

pub struct TokenizerStateIterator<T: Iterator<Item = char>> {
    chars: T,
    cache: VecDeque<JsonToken>,
    state: TokenizerState,
    end: bool,
}

impl<T: Iterator<Item = char>> TokenizerStateIterator<T> {
    pub fn new(chars: T) -> Self {
        Self {
            chars,
            cache: default(),
            state: default(),
            end: false,
        }
    }
}

impl<T: Iterator<Item = char>> Iterator for TokenizerStateIterator<T> {
    type Item = JsonToken;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(result) = self.cache.pop_front() {
                return Some(result);
            }
            if self.end {
                return None;
            }
            match self.chars.next() {
                Some(c) => self.cache.extend(self.state.push_mut(c)),
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
