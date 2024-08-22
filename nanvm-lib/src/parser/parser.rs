use super::shared::{
    AnyResult, AnyState, AnyStateExtension, ConstState, JsonState, ParseError, ParseResult,
    ParsingStatus, RootState, RootStatus,
};
use super::{
    path::{concat, split},
    shared::JsonElement,
};
use crate::{
    common::default::default,
    js::any::Any,
    mem::manager::{Dealloc, Manager},
    tokenizer::{tokenize, JsonToken},
};
use io_trait::Io;
use std::collections::{BTreeMap, BTreeSet};

pub struct ModuleCache<D: Dealloc> {
    pub complete: BTreeMap<String, Any<D>>,
    pub progress: BTreeSet<String>,
}

impl<D: Dealloc> Default for ModuleCache<D> {
    fn default() -> Self {
        Self {
            complete: default(),
            progress: default(),
        }
    }
}

pub struct Context<'a, M: Manager, I: Io> {
    manager: M,
    io: &'a I,
    path: String,
    module_cache: &'a mut ModuleCache<M::Dealloc>,
}

impl<'a, M: Manager, I: Io> Context<'a, M, I> {
    pub fn new(
        manager: M,
        io: &'a I,
        path: String,
        module_cache: &'a mut ModuleCache<M::Dealloc>,
    ) -> Self {
        Context {
            manager,
            io,
            path,
            module_cache,
        }
    }
}

fn root_state_parse<M: Manager, I: Io>(
    mut root_state: RootState<M::Dealloc>,
    context: &mut Context<M, I>,
    token: JsonToken,
) -> JsonState<M::Dealloc> {
    match root_state.status {
        RootStatus::Initial => match token {
            JsonToken::NewLine => JsonState::ParseRoot(RootState {
                status: RootStatus::Initial,
                state: root_state.state,
                new_line: true,
            }),
            JsonToken::Id(s) => match root_state.new_line {
                true => match s.as_ref() {
                    "const" => JsonState::ParseRoot(RootState {
                        status: RootStatus::Const,
                        state: root_state.state.set_djs(),
                        new_line: false,
                    }),
                    "export" if root_state.state.data_type.is_mjs_compatible() => {
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::Export,
                            state: root_state.state.set_mjs(),
                            new_line: false,
                        })
                    }
                    "module" if root_state.state.data_type.is_cjs_compatible() => {
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::Module,
                            state: root_state.state.set_cjs(),
                            new_line: false,
                        })
                    }
                    "import" if root_state.state.data_type.is_mjs_compatible() => {
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::Import,
                            state: root_state.state.set_mjs(),
                            new_line: false,
                        })
                    }
                    _ => any_state_parse_for_module(root_state.state, context, JsonToken::Id(s)),
                },
                false => JsonState::Error(ParseError::NewLineExpected),
            },
            _ => match root_state.new_line {
                true => any_state_parse_for_module(root_state.state, context, token),
                false => JsonState::Error(ParseError::NewLineExpected),
            },
        },
        RootStatus::Export => match token {
            JsonToken::Id(s) => match s.as_ref() {
                "default" => JsonState::ParseModule(root_state.state),
                _ => JsonState::Error(ParseError::WrongExportStatement),
            },
            _ => JsonState::Error(ParseError::WrongExportStatement),
        },
        RootStatus::Module => match token {
            JsonToken::Dot => JsonState::ParseRoot(RootState {
                status: RootStatus::ModuleDot,
                state: root_state.state,
                new_line: false,
            }),
            _ => JsonState::Error(ParseError::WrongExportStatement),
        },
        RootStatus::ModuleDot => match token {
            JsonToken::Id(s) => match s.as_ref() {
                "exports" => JsonState::ParseRoot(RootState {
                    status: RootStatus::ModuleDotExports,
                    state: root_state.state,
                    new_line: false,
                }),
                _ => JsonState::Error(ParseError::WrongExportStatement),
            },
            _ => JsonState::Error(ParseError::WrongExportStatement),
        },
        RootStatus::ModuleDotExports => match token {
            JsonToken::Equals => JsonState::ParseModule(root_state.state),
            _ => JsonState::Error(ParseError::WrongExportStatement),
        },
        RootStatus::Const => match token {
            JsonToken::Id(s) => JsonState::ParseRoot(RootState {
                status: RootStatus::ConstId(s),
                state: root_state.state,
                new_line: false,
            }),
            _ => JsonState::Error(ParseError::WrongConstStatement),
        },
        RootStatus::ConstId(s) => match token {
            JsonToken::Equals => JsonState::ParseConst(ConstState {
                key: s,
                state: root_state.state,
            }),
            _ => JsonState::Error(ParseError::WrongConstStatement),
        },
        RootStatus::Import => match token {
            JsonToken::Id(s) => JsonState::ParseRoot(RootState {
                status: RootStatus::ImportId(s),
                state: root_state.state,
                new_line: false,
            }),
            _ => JsonState::Error(ParseError::WrongImportStatement),
        },
        RootStatus::ImportId(id) => match token {
            JsonToken::Id(s) => match s.as_ref() {
                "from" => JsonState::ParseRoot(RootState {
                    status: RootStatus::ImportIdFrom(id),
                    state: root_state.state,
                    new_line: false,
                }),
                _ => JsonState::Error(ParseError::WrongImportStatement),
            },
            _ => JsonState::Error(ParseError::WrongImportStatement),
        },
        RootStatus::ImportIdFrom(id) => match token {
            JsonToken::String(s) => {
                let current_path = concat(split(&context.path).0, s.as_str());
                if let Some(any) = context.module_cache.complete.get(&current_path) {
                    root_state.state.consts.insert(id, any.clone());
                    return JsonState::ParseRoot(RootState {
                        status: RootStatus::Initial,
                        state: root_state.state,
                        new_line: false,
                    });
                }
                if context.module_cache.progress.contains(&current_path) {
                    return JsonState::Error(ParseError::CircularDependency);
                }
                context.module_cache.progress.insert(current_path.clone());
                let read_result = context.io.read_to_string(current_path.as_str());
                match read_result {
                    Ok(s) => {
                        let tokens = tokenize(s);
                        let res = parse_with_tokens(context, tokens.into_iter());
                        match res {
                            Ok(r) => {
                                context.module_cache.progress.remove(&current_path);
                                context
                                    .module_cache
                                    .complete
                                    .insert(current_path, r.any.clone());
                                root_state.state.consts.insert(id, r.any);
                                JsonState::ParseRoot(RootState {
                                    status: RootStatus::Initial,
                                    state: root_state.state,
                                    new_line: false,
                                })
                            }
                            Err(e) => JsonState::Error(e),
                        }
                    }
                    Err(_) => JsonState::Error(ParseError::CannotReadFile),
                }
            }
            _ => JsonState::Error(ParseError::WrongImportStatement),
        },
    }
}

fn const_state_parse<M: Manager, I: Io>(
    const_state: ConstState<M::Dealloc>,
    context: &mut Context<M, I>,
    token: JsonToken,
) -> JsonState<M::Dealloc> {
    match token {
        JsonToken::Semicolon => todo!(),
        _ => {
            let result = any_state_parse(const_state.state, context, token);
            match result {
                AnyResult::Continue(state) => JsonState::ParseConst(ConstState {
                    key: const_state.key,
                    state,
                }),
                AnyResult::Success(mut success) => {
                    success.state.consts.insert(const_state.key, success.value);
                    JsonState::ParseRoot(RootState {
                        status: RootStatus::Initial,
                        state: success.state,
                        new_line: false,
                    })
                }
                AnyResult::Error(error) => JsonState::Error(error),
            }
        }
    }
}

fn any_state_parse_for_module<M: Manager, I: Io>(
    any_state: AnyState<M::Dealloc>,
    context: &mut Context<M, I>,
    token: JsonToken,
) -> JsonState<M::Dealloc> {
    let result = any_state_parse(any_state, context, token);
    match result {
        AnyResult::Continue(state) => JsonState::ParseModule(state),
        AnyResult::Success(success) => JsonState::Result(ParseResult {
            data_type: success.state.data_type,
            any: success.value,
        }),
        AnyResult::Error(error) => JsonState::Error(error),
    }
}

fn any_state_parse_import_value<M: Manager, I: Io>(
    any_state: AnyState<M::Dealloc>,
    context: &mut Context<M, I>,
    token: JsonToken,
) -> AnyResult<M::Dealloc> {
    match token {
        JsonToken::String(s) => {
            let current_path = concat(split(&context.path).0, s.as_str());
            if let Some(any) = context.module_cache.complete.get(&current_path) {
                return AnyResult::Continue(AnyState {
                    status: ParsingStatus::ImportEnd,
                    current: JsonElement::Any(any.clone()),
                    ..any_state
                });
            }
            if context.module_cache.progress.contains(&current_path) {
                return AnyResult::Error(ParseError::CircularDependency);
            }
            context.module_cache.progress.insert(current_path.clone());
            let read_result = context.io.read_to_string(current_path.as_str());
            match read_result {
                Ok(s) => {
                    let tokens = tokenize(s);
                    let res = parse_with_tokens(context, tokens.into_iter());
                    match res {
                        Ok(r) => {
                            context.module_cache.progress.remove(&current_path);
                            context
                                .module_cache
                                .complete
                                .insert(current_path, r.any.clone());
                            AnyResult::Continue(AnyState {
                                status: ParsingStatus::ImportEnd,
                                current: JsonElement::Any(r.any),
                                ..any_state
                            })
                        }
                        Err(e) => AnyResult::Error(e),
                    }
                }
                Err(_) => AnyResult::<M::Dealloc>::Error(ParseError::CannotReadFile),
            }
        }
        _ => AnyResult::Error(ParseError::WrongRequireStatement),
    }
}

fn any_state_parse<M: Manager, I: Io>(
    any_state: AnyState<M::Dealloc>,
    context: &mut Context<M, I>,
    token: JsonToken,
) -> AnyResult<M::Dealloc> {
    match any_state.status {
        ParsingStatus::Initial | ParsingStatus::ObjectColon => {
            any_state.parse_value(context.manager, token)
        }
        ParsingStatus::ArrayBegin => any_state.parse_array_begin(context.manager, token),
        ParsingStatus::ArrayValue => any_state.parse_array_value(context.manager, token),
        ParsingStatus::ArrayComma => any_state.parse_array_comma(context.manager, token),
        ParsingStatus::ObjectBegin => any_state.parse_object_begin(context.manager, token),
        ParsingStatus::ObjectKey => any_state.parse_object_key(token),
        ParsingStatus::ObjectValue => any_state.parse_object_next(context.manager, token),
        ParsingStatus::ObjectComma => any_state.parse_object_comma(context.manager, token),
        ParsingStatus::ImportBegin => any_state.parse_import_begin(token),
        ParsingStatus::ImportValue => any_state_parse_import_value(any_state, context, token),
        ParsingStatus::ImportEnd => any_state.parse_import_end(token),
    }
}

fn json_state_push<M: Manager, I: Io>(
    json_state: JsonState<M::Dealloc>,
    context: &mut Context<M, I>,
    token: JsonToken,
) -> JsonState<M::Dealloc> {
    if token == JsonToken::NewLine {
        return match json_state {
            JsonState::ParseRoot(state) => root_state_parse(state, context, token),
            _ => json_state,
        };
    }
    match json_state {
        JsonState::ParseRoot(state) => root_state_parse(state, context, token),
        JsonState::Result(_) => JsonState::Error(ParseError::UnexpectedToken),
        JsonState::ParseModule(state) => any_state_parse_for_module(state, context, token),
        JsonState::ParseConst(state) => const_state_parse(state, context, token),
        _ => json_state,
    }
}

pub fn parse<M: Manager, I: Io>(
    context: &mut Context<M, I>,
) -> Result<ParseResult<M::Dealloc>, ParseError> {
    context.module_cache.progress.insert(context.path.clone());
    let read_result = context.io.read_to_string(context.path.as_str());
    match read_result {
        Ok(s) => {
            let tokens = tokenize(s);
            parse_with_tokens(context, tokens.into_iter())
        }
        Err(_) => Err(ParseError::CannotReadFile),
    }
}

pub fn parse_with_tokens<M: Manager, I: Io>(
    context: &mut Context<M, I>,
    iter: impl Iterator<Item = JsonToken>,
) -> Result<ParseResult<M::Dealloc>, ParseError> {
    let mut state = JsonState::ParseRoot(RootState {
        status: RootStatus::Initial,
        state: default(),
        new_line: true,
    });
    for token in iter {
        state = json_state_push(state, context, token);
    }
    state.end()
}

#[cfg(test)]
mod test {
    use io_test::VirtualIo;
    use io_trait::Io;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        common::default::default,
        js::{js_array::JsArrayRef, js_object::JsObjectRef, js_string::JsStringRef, type_::Type},
        mem::{global::GLOBAL, local::Local, manager::Manager},
        tokenizer::{tokenize, ErrorType, JsonToken},
    };

    use super::super::{parser::parse, path::concat, shared::DataType};

    use super::{parse_with_tokens, Context, ModuleCache, ParseError, ParseResult};

    fn virtual_io() -> VirtualIo {
        VirtualIo::new(&[])
    }

    fn create_test_context<'a, M: Manager>(
        manager: M,
        io: &'a VirtualIo,
        module_cache: &'a mut ModuleCache<M::Dealloc>,
    ) -> Context<'a, M, VirtualIo> {
        Context::new(manager, io, default(), module_cache)
    }

    fn parse_with_virtual_io<M: Manager>(
        manager: M,
        iter: impl Iterator<Item = JsonToken>,
    ) -> Result<ParseResult<M::Dealloc>, ParseError> {
        parse_with_tokens(
            &mut create_test_context(manager, &virtual_io(), &mut default()),
            iter,
        )
    }

    fn test_local() {
        let local = Local::default();
        let _ = parse_with_tokens(
            &mut create_test_context(&local, &virtual_io(), &mut default()),
            [].into_iter(),
        );
    }

    fn test_global() {
        let _ = {
            let global = GLOBAL;
            parse_with_tokens(
                &mut create_test_context(global, &virtual_io(), &mut default()),
                [].into_iter(),
            )
        };
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_json() {
        let json_str = include_str!("../../test/test-json.json");
        let tokens = tokenize(json_str.to_owned());
        let local = Local::default();
        let result = parse_with_virtual_io(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Json);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_djs() {
        let json_str = include_str!("../../test/test-djs.d.cjs");
        let tokens = tokenize(json_str.to_owned());
        let local = Local::default();
        let result = parse_with_virtual_io(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Cjs);

        let json_str = include_str!("../../test/test-djs.d.mjs");
        let tokens = tokenize(json_str.to_owned());
        let local = Local::default();
        let result = parse_with_virtual_io(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Mjs);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_const() {
        let local = Local::default();
        test_const_with_manager(&local);
    }

    fn test_const_with_manager<M: Manager>(manager: M) {
        let json_str = include_str!("../../test/test-const.d.cjs");
        let tokens = tokenize(json_str.to_owned());
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.try_move(), Ok(2.0));
        let item1 = items[1].clone();
        assert_eq!(item1.try_move(), Ok(2.0));

        let json_str = include_str!("../../test/test-const-error.d.cjs.txt");
        let tokens = tokenize(json_str.to_owned());
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let json_str = include_str!("../../test/test-const-error-new-line.d.cjs.txt");
        let tokens = tokenize(json_str.to_owned());
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::NewLineExpected);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_stack() {
        let local = Local::default();
        test_stack_with_manager(&local);
    }

    fn test_stack_with_manager<M: Manager>(manager: M) {
        let json_str = include_str!("../../test/test-stack.d.cjs");
        let tokens = tokenize(json_str.to_owned());
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        let result_unwrap = item0.try_move::<JsObjectRef<M::Dealloc>>().unwrap();
        let items = result_unwrap.items();
        let (key0, value0) = items[0].clone();
        let key0_items = key0.items();
        assert_eq!(key0_items, [0x61]);
        let result_unwrap = value0.try_move::<JsArrayRef<M::Dealloc>>().unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.get_type(), Type::Null);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_import() {
        let local = Local::default();
        test_import_with_manager(&local);
    }

    fn test_import_with_manager<M: Manager>(manager: M) {
        let io: VirtualIo = VirtualIo::new(&[]);

        let main = include_str!("../../test/test_import_main.d.cjs");
        //let path = "../../test/test-import-main.d.cjs";
        let main_path = "test_import_main.d.cjs";
        io.write(main_path, main.as_bytes()).unwrap();

        let module = include_str!("../../test/test_import_module.d.cjs");
        let module_path = "test_import_module.d.cjs";
        io.write(module_path, module.as_bytes()).unwrap();

        let mut mc = default();
        let mut context = Context::new(
            manager,
            &io,
            concat(io.current_dir().unwrap().as_str(), main_path),
            &mut mc,
        );

        let result = parse(&mut context);
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.try_move(), Ok(3.0));

        let io: VirtualIo = VirtualIo::new(&[]);

        let main = include_str!("../../test/test_import_main.d.mjs");
        //let path = "../../test/test-import-main.d.mjs";
        let main_path = "test_import_main.d.mjs";
        io.write(main_path, main.as_bytes()).unwrap();

        let module = include_str!("../../test/test_import_module.d.mjs");
        let module_path = "test_import_module.d.mjs";
        io.write(module_path, module.as_bytes()).unwrap();

        let mut mc = default();
        let mut context = Context::new(
            manager,
            &io,
            concat(io.current_dir().unwrap().as_str(), main_path),
            &mut mc,
        );

        let result = parse(&mut context);
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.try_move(), Ok(4.0));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_cache() {
        let local = Local::default();
        test_cache_with_manager(&local);
    }

    fn test_cache_with_manager<M: Manager>(manager: M) {
        let io: VirtualIo = VirtualIo::new(&[]);

        let main = include_str!("../../test/test_cache_main.d.cjs");
        let main_path = "test_cache_main.d.cjs";
        io.write(main_path, main.as_bytes()).unwrap();

        let module_b = include_str!("../../test/test_cache_b.d.cjs");
        let module_b_path = "test_cache_b.d.cjs";
        io.write(module_b_path, module_b.as_bytes()).unwrap();

        let module_c = include_str!("../../test/test_cache_c.d.cjs");
        let module_c_path = "test_cache_c.d.cjs";
        io.write(module_c_path, module_c.as_bytes()).unwrap();

        let mut mc = default();
        let mut context = Context::new(
            manager,
            &io,
            concat(io.current_dir().unwrap().as_str(), main_path),
            &mut mc,
        );

        let result = parse(&mut context);
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.try_move(), Ok(1.0));
        let item1 = items[1].clone();
        assert_eq!(item1.try_move(), Ok(1.0));

        let io: VirtualIo = VirtualIo::new(&[]);

        let main = include_str!("../../test/test_cache_main.d.mjs");
        let main_path = "test_cache_main.d.mjs";
        io.write(main_path, main.as_bytes()).unwrap();

        let module_b = include_str!("../../test/test_cache_b.d.mjs");
        let module_b_path = "test_cache_b.d.mjs";
        io.write(module_b_path, module_b.as_bytes()).unwrap();

        let module_c = include_str!("../../test/test_cache_c.d.mjs");
        let module_c_path = "test_cache_c.d.mjs";
        io.write(module_c_path, module_c.as_bytes()).unwrap();

        let mut mc = default();
        let mut context = Context::new(
            manager,
            &io,
            concat(io.current_dir().unwrap().as_str(), main_path),
            &mut mc,
        );

        let result = parse(&mut context);
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.try_move(), Ok(2.0));
        let item1 = items[1].clone();
        assert_eq!(item1.try_move(), Ok(2.0));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_circular_error() {
        let local = Local::default();
        test_circular_error_with_manager(&local);
    }

    fn test_circular_error_with_manager<M: Manager>(manager: M) {
        let io: VirtualIo = VirtualIo::new(&[]);

        let main = include_str!("../../test/test_circular_1.d.cjs.txt");
        let main_path = "test_circular_1.d.cjs.txt";
        io.write(main_path, main.as_bytes()).unwrap();

        let module = include_str!("../../test/test_circular_2.d.cjs.txt");
        let module_path = "test_circular_2.d.cjs.txt";
        io.write(module_path, module.as_bytes()).unwrap();

        let mut mc = default();
        let mut context = Context::new(
            manager,
            &io,
            concat(io.current_dir().unwrap().as_str(), main_path),
            &mut mc,
        );

        let result = parse(&mut context);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::CircularDependency);

        let io: VirtualIo = VirtualIo::new(&[]);

        let main = include_str!("../../test/test_circular_1.d.mjs.txt");
        let main_path = "test_circular_1.d.mjs.txt";
        io.write(main_path, main.as_bytes()).unwrap();

        let module = include_str!("../../test/test_circular_2.d.mjs.txt");
        let module_path = "test_circular_2.d.mjs.txt";
        io.write(module_path, module.as_bytes()).unwrap();

        let mut mc = default();
        let mut context = Context::new(
            manager,
            &io,
            concat(io.current_dir().unwrap().as_str(), main_path),
            &mut mc,
        );

        let result = parse(&mut context);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::CircularDependency);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_import_error() {
        let local = Local::default();
        test_import_error_with_manager(&local);
    }

    fn test_import_error_with_manager<M: Manager>(manager: M) {
        let io: VirtualIo = VirtualIo::new(&[]);

        let main = include_str!("../../test/test_import_error.d.cjs.txt");
        //let path = "../../test/test-import-main.d.cjs";
        let main_path = "test_import_error.d.cjs.txt";
        io.write(main_path, main.as_bytes()).unwrap();

        let module = include_str!("../../test/test_import_module.d.mjs");
        let module_path = "test_import_module.d.mjs";
        io.write(module_path, module.as_bytes()).unwrap();

        let mut mc = default();
        let mut context = Context::new(
            manager,
            &io,
            concat(io.current_dir().unwrap().as_str(), main_path),
            &mut mc,
        );

        let result = parse(&mut context);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::UnexpectedToken);

        let io: VirtualIo = VirtualIo::new(&[]);

        let main = include_str!("../../test/test_import_error.d.mjs.txt");
        //let path = "../../test/test-import-main.d.cjs";
        let main_path = "test_import_error.d.mjs.txt";
        io.write(main_path, main.as_bytes()).unwrap();

        let module = include_str!("../../test/test_import_module.d.cjs");
        let module_path = "test_import_module.d.cjs";
        io.write(module_path, module.as_bytes()).unwrap();

        let mut mc = default();
        let mut context = Context::new(
            manager,
            &io,
            concat(io.current_dir().unwrap().as_str(), main_path),
            &mut mc,
        );

        let result = parse(&mut context);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::UnexpectedToken);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_trailing_comma() {
        let local = Local::default();
        let json_str = include_str!("../../test/test-trailing-comma.d.cjs");
        let tokens = tokenize(json_str.to_owned());
        let result = parse_with_virtual_io(&local, tokens.into_iter());
        assert!(result.is_ok());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_check_sizes() {
        let local = Local::default();
        {
            let tokens = [
                JsonToken::ObjectBegin,
                JsonToken::String(String::from("k")),
                JsonToken::Colon,
                JsonToken::ObjectBegin,
                JsonToken::ObjectEnd,
                JsonToken::ObjectEnd,
            ];
            {
                let result = parse_with_virtual_io(&local, tokens.into_iter());
                assert!(result.is_ok());
                let _result_unwrap = result.unwrap();
            }
            assert_eq!(local.size(), 0);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_check_sizes2() {
        let local = Local::default();
        {
            let tokens = [
                JsonToken::ObjectBegin,
                JsonToken::String(String::from("k")),
                JsonToken::Colon,
                JsonToken::ObjectBegin,
                JsonToken::ObjectEnd,
                JsonToken::ObjectEnd,
            ];
            {
                let result = parse_with_virtual_io(&local, tokens.into_iter());
                assert!(result.is_ok());
                let result_unwrap = result.unwrap().any;
                let _result_unwrap = result_unwrap.try_move::<JsObjectRef<_>>();
            }
            assert_eq!(local.size(), 0);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_data_type() {
        let local = Local::default();
        let tokens = [JsonToken::Id(String::from("null"))];
        let result = parse_with_virtual_io(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Json);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_export_block() {
        let local = Local::default();
        let tokens = [
            JsonToken::Id(String::from("export")),
            JsonToken::Id(String::from("default")),
            JsonToken::Id(String::from("null")),
        ];
        let result = parse_with_virtual_io(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Mjs);

        let local = Local::default();
        let tokens = [
            JsonToken::Id(String::from("module")),
            JsonToken::Dot,
            JsonToken::Id(String::from("exports")),
            JsonToken::Equals,
            JsonToken::Id(String::from("null")),
        ];
        let result = parse_with_virtual_io(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Cjs);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_id_in_objects() {
        let local = Local::default();
        let tokens = [
            JsonToken::Id(String::from("export")),
            JsonToken::Id(String::from("default")),
            JsonToken::ObjectBegin,
            JsonToken::Id(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(&local, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result.unwrap().any.try_move::<JsObjectRef<_>>().unwrap();
        let items = result_unwrap.items();
        let (key0, value0) = items[0].clone();
        let key0_items = key0.items();
        assert_eq!(key0_items, [0x6b, 0x65, 0x79]);
        assert_eq!(value0.try_move(), Ok(0.0));

        let local = Local::default();
        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::Id(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(&local, tokens.into_iter());
        assert!(result.is_err());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_valid_local() {
        test_valid_with_manager(&Local::default());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_valid_global() {
        test_valid_with_manager(GLOBAL);
    }

    fn test_valid_with_manager<M: Manager>(manager: M) {
        let tokens = [JsonToken::Id(String::from("null"))];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().any.get_type(), Type::Null);

        let tokens = [JsonToken::Id(String::from("true"))];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().any.try_move(), Ok(true));

        let tokens = [JsonToken::Id(String::from("false"))];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().any.try_move(), Ok(false));

        let tokens = [JsonToken::Number(0.1)];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().any.try_move(), Ok(0.1));

        let tokens = [JsonToken::String(String::from("abc"))];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result = result.unwrap().any.try_move::<JsStringRef<M::Dealloc>>();
        assert!(result.is_ok());
        let result = result.unwrap();
        let items = result.items();
        assert_eq!(items, [0x61, 0x62, 0x63]);

        let tokens = [JsonToken::ArrayBegin, JsonToken::ArrayEnd];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        assert!(items.is_empty());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Number(1.0),
            JsonToken::Comma,
            JsonToken::Id(String::from("true")),
            JsonToken::ArrayEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.try_move(), Ok(1.0));
        let item1 = items[1].clone();
        assert_eq!(item1.try_move(), Ok(true));

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Number(1.0),
            JsonToken::Comma,
            JsonToken::Id(String::from("true")),
            JsonToken::Comma,
            JsonToken::ArrayEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("k1")),
            JsonToken::Colon,
            JsonToken::Number(1.0),
            JsonToken::Comma,
            JsonToken::String(String::from("k0")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::Comma,
            JsonToken::String(String::from("k2")),
            JsonToken::Colon,
            JsonToken::Number(2.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsObjectRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let (key0, value0) = items[0].clone();
        let key0_items = key0.items();
        assert_eq!(key0_items, [0x6b, 0x30]);
        assert_eq!(value0.try_move(), Ok(0.0));
        let (key1, value1) = items[1].clone();
        let key1_items = key1.items();
        assert_eq!(key1_items, [0x6b, 0x31]);
        assert_eq!(value1.try_move(), Ok(1.0));
        let (key2, value2) = items[2].clone();
        let key2_items = key2.items();
        assert_eq!(key2_items, [0x6b, 0x32]);
        assert_eq!(value2.try_move(), Ok(2.0));

        let tokens = [JsonToken::ObjectBegin, JsonToken::ObjectEnd];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsObjectRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        assert!(items.is_empty());
        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("k")),
            JsonToken::Colon,
            JsonToken::ObjectBegin,
            JsonToken::ObjectEnd,
            JsonToken::ObjectEnd,
        ];
        {
            let result = parse_with_virtual_io(manager, tokens.into_iter());
            assert!(result.is_ok());
            let result_unwrap = result.unwrap();
            let result_unwrap = result_unwrap
                .any
                .try_move::<JsObjectRef<M::Dealloc>>()
                .unwrap();
            let items = result_unwrap.items();
            let (_, value0) = items[0].clone();
            let value0_unwrap = value0.try_move::<JsObjectRef<M::Dealloc>>().unwrap();
            let value0_items = value0_unwrap.items();
            assert!(value0_items.is_empty());
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_invalid_local() {
        test_invalid_with_manager(&Local::default());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_invalid_global() {
        test_invalid_with_manager(GLOBAL);
    }

    fn test_invalid_with_manager<M: Manager>(manager: M) {
        let tokens = [];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ErrorToken(ErrorType::InvalidNumber)];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ArrayBegin, JsonToken::Comma, JsonToken::ArrayEnd];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Number(0.0),
            JsonToken::Number(1.0),
            JsonToken::ArrayEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Number(0.0),
            JsonToken::Comma,
            JsonToken::Comma,
            JsonToken::Number(1.0),
            JsonToken::ArrayEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::ArrayEnd,
            JsonToken::ArrayEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ArrayBegin, JsonToken::String(String::default())];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Comma,
            JsonToken::Number(1.0),
            JsonToken::ArrayEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ArrayBegin, JsonToken::Colon, JsonToken::ArrayEnd];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ArrayEnd];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::Comma,
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::Number(0.0),
            JsonToken::Comma,
            JsonToken::Number(1.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("key")),
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("key")),
            JsonToken::Colon,
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("key0")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::Comma,
            JsonToken::Comma,
            JsonToken::String(String::from("key1")),
            JsonToken::Colon,
            JsonToken::Number(1.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::ObjectEnd,
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::Comma,
            JsonToken::String(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ObjectEnd];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::ObjectBegin,
            JsonToken::ArrayEnd,
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::ArrayBegin,
            JsonToken::ObjectEnd,
            JsonToken::ArrayEnd,
        ];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());
    }
}
