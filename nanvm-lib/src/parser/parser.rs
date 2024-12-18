use super::{
    any_state::{AnyResult, AnyState},
    const_state::ConstState,
    json_state::JsonState,
    path::{concat, split},
    root_state::{RootState, RootStatus},
    shared::{JsonElement, ModuleCache, ParseError, ParseResult, ParsingStatus},
};
use crate::{
    common::default::default,
    mem::manager::Manager,
    tokenizer::{tokenize, JsonToken},
};
use io_trait::Io;

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

fn const_state_parse<M: Manager + 'static, I: Io>(
    const_state: ConstState<M>,
    context: &mut Context<M, I>,
    token: JsonToken<M::Dealloc>,
) -> JsonState<M> {
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

fn any_state_parse_for_module<M: Manager + 'static, I: Io>(
    any_state: AnyState<M>,
    context: &mut Context<M, I>,
    token: JsonToken<M::Dealloc>,
) -> JsonState<M> {
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

fn any_state_parse_import_value<M: Manager + 'static, I: Io>(
    any_state: AnyState<M>,
    context: &mut Context<M, I>,
    token: JsonToken<M::Dealloc>,
) -> AnyResult<M> {
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
                    let tokens = tokenize(context.manager, s);
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
                Err(_) => AnyResult::<M>::Error(ParseError::CannotReadFile),
            }
        }
        _ => AnyResult::Error(ParseError::WrongRequireStatement),
    }
}

fn any_state_parse<M: Manager + 'static, I: Io>(
    any_state: AnyState<M>,
    context: &mut Context<M, I>,
    token: JsonToken<M::Dealloc>,
) -> AnyResult<M> {
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

fn root_state_parse<M: Manager + 'static, I: Io>(
    root_state: RootState<M>,
    context: &mut Context<M, I>,
    token: JsonToken<M::Dealloc>,
) -> JsonState<M> {
    let (json_state, import) = root_state.parse(context.manager, token);
    match import {
        None => json_state,
        Some((id, module)) => match json_state {
            JsonState::ParseRoot(mut root_state) => {
                let current_path = concat(split(&context.path).0, module.as_str());
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
                        let tokens = tokenize(context.manager, s);
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
            _ => panic!("JsonState::ParseRoot expected when root_state.parse returns import"),
        },
    }
}

fn json_state_push<M: Manager + 'static, I: Io>(
    json_state: JsonState<M>,
    context: &mut Context<M, I>,
    token: JsonToken<M::Dealloc>,
) -> JsonState<M> {
    if let JsonToken::NewLine = token {
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

pub fn parse<M: Manager + 'static, I: Io>(
    context: &mut Context<M, I>,
) -> Result<ParseResult<M::Dealloc>, ParseError> {
    context.module_cache.progress.insert(context.path.clone());
    let read_result = context.io.read_to_string(context.path.as_str());
    match read_result {
        Ok(s) => {
            let tokens = tokenize(context.manager, s);
            parse_with_tokens(context, tokens.into_iter())
        }
        Err(_) => Err(ParseError::CannotReadFile),
    }
}

pub fn parse_with_tokens<M: Manager + 'static, I: Io>(
    context: &mut Context<M, I>,
    iter: impl Iterator<Item = JsonToken<M::Dealloc>>,
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
        js::{
            js_array::JsArrayRef,
            js_bigint::{from_u64, new_bigint, JsBigintRef, Sign},
            js_object::JsObjectRef,
            js_string::JsStringRef,
            type_::Type,
        },
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

    fn parse_with_virtual_io<M: Manager + 'static>(
        manager: M,
        iter: impl Iterator<Item = JsonToken<M::Dealloc>>,
    ) -> Result<ParseResult<M::Dealloc>, ParseError> {
        parse_with_tokens(
            &mut create_test_context(manager, &virtual_io(), &mut default()),
            iter,
        )
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
        let tokens = tokenize(GLOBAL, json_str.to_owned());
        let result = parse_with_virtual_io(GLOBAL, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Json);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_djs() {
        let json_str = include_str!("../../test/test-djs.d.cjs");
        let tokens = tokenize(GLOBAL, json_str.to_owned());
        let result = parse_with_virtual_io(GLOBAL, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Cjs);

        let json_str = include_str!("../../test/test-djs.d.mjs");
        let tokens = tokenize(GLOBAL, json_str.to_owned());
        let result = parse_with_virtual_io(GLOBAL, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Mjs);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_const() {
        test_const_with_manager(GLOBAL);
    }

    fn test_const_with_manager<M: Manager + 'static>(manager: M) {
        let json_str = include_str!("../../test/test-const.d.cjs");
        let tokens = tokenize(manager, json_str.to_owned());
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
        let tokens = tokenize(manager, json_str.to_owned());
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let json_str = include_str!("../../test/test-const-error-new-line.d.cjs.txt");
        let tokens = tokenize(manager, json_str.to_owned());
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::NewLineExpected);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_stack() {
        test_stack_with_manager(GLOBAL);
    }

    fn test_stack_with_manager<M: Manager + 'static>(manager: M) {
        let json_str = include_str!("../../test/test-stack.d.cjs");
        let tokens = tokenize(manager, json_str.to_owned());
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
        test_import_with_manager(GLOBAL);
    }

    fn test_import_with_manager<M: Manager + 'static>(manager: M) {
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
        test_cache_with_manager(GLOBAL);
    }

    fn test_cache_with_manager<M: Manager + 'static>(manager: M) {
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
        test_circular_error_with_manager(GLOBAL);
    }

    fn test_circular_error_with_manager<M: Manager + 'static>(manager: M) {
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
        test_import_error_with_manager(GLOBAL);
    }

    fn test_import_error_with_manager<M: Manager + 'static>(manager: M) {
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
        let json_str = include_str!("../../test/test-trailing-comma.d.cjs");
        let tokens = tokenize(GLOBAL, json_str.to_owned());
        let result = parse_with_virtual_io(GLOBAL, tokens.into_iter());
        assert!(result.is_ok());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_check_sizes() {
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
                let result = parse_with_virtual_io(GLOBAL, tokens.into_iter());
                assert!(result.is_ok());
                let _result_unwrap = result.unwrap();
            }
            //assert_eq!(GLOBAL.size(), 0);
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
                let result = parse_with_virtual_io(GLOBAL, tokens.into_iter());
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
        let tokens = [JsonToken::Id(String::from("null"))];
        let result = parse_with_virtual_io(GLOBAL, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Json);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_export_block() {
        let tokens = [
            JsonToken::Id(String::from("export")),
            JsonToken::Id(String::from("default")),
            JsonToken::Id(String::from("null")),
        ];
        let result = parse_with_virtual_io(GLOBAL, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Mjs);

        let tokens = [
            JsonToken::Id(String::from("module")),
            JsonToken::Dot,
            JsonToken::Id(String::from("exports")),
            JsonToken::Equals,
            JsonToken::Id(String::from("null")),
        ];
        let result = parse_with_virtual_io(GLOBAL, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Cjs);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_id_in_objects() {
        let tokens = [
            JsonToken::Id(String::from("export")),
            JsonToken::Id(String::from("default")),
            JsonToken::ObjectBegin,
            JsonToken::Id(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(GLOBAL, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result.unwrap().any.try_move::<JsObjectRef<_>>().unwrap();
        let items = result_unwrap.items();
        let (key0, value0) = items[0].clone();
        let key0_items = key0.items();
        assert_eq!(key0_items, [0x6b, 0x65, 0x79]);
        assert_eq!(value0.try_move(), Ok(0.0));

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::Id(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virtual_io(GLOBAL, tokens.into_iter());
        assert!(result.is_err());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_valid_global() {
        test_valid_with_manager(GLOBAL);
    }

    fn test_valid_with_manager<M: Manager + 'static>(manager: M) {
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

        let tokens = [JsonToken::BigInt(from_u64(manager, Sign::Positive, 1))];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result = result.unwrap().any.try_move::<JsBigintRef<M::Dealloc>>();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.header_len(), 1);
        let items = result.items();
        assert_eq!(items, [0x1]);

        let tokens = [JsonToken::BigInt(new_bigint(
            manager,
            Sign::Negative,
            [2, 3],
        ))];
        let result = parse_with_virtual_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result = result.unwrap().any.try_move::<JsBigintRef<M::Dealloc>>();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.header_len(), -2);
        let items = result.items();
        assert_eq!(items, [0x2, 0x3]);

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
    fn test_invalid_global() {
        test_invalid_with_manager(GLOBAL);
    }

    fn test_invalid_with_manager<M: Manager + 'static>(manager: M) {
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
