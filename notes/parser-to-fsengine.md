The purpose of this note is to discuss NaNVM internals separating parser and fsengine (that's the
current name of NaNVM execution engine). To shorten paths to MVP we plan to experiment with an initial
`fsengine` based off an AST produced by the parser - as opposed to compiling to bytecode (that we
rather postpone). Below we use `lazy loader` term to distinguish that logic from the current
eagerly-loading parser logic.

See [bytecode-initial.md](bytecode-initial.md) on an `AST` draft: with that approach parser, given a module text, produces
`ast::Module` which later can be processed in various ways (as opposed to the current eager loading
of a DAG of modules):
- The lazy-loader involves asynchronicity at selected APIs - which improves abilities to reason about
what's eager, what's IO / async - dependent in the engine. Other components (parser, fsengine) don't
have intrinsic IO interactions (though in fsengine IO interactions can be executed via calls into
host's intrinsics).
- The `ast::Module` result of one module parsing process, as well as results of processing of
several interconnected modules (by the lazy loader), can be used directly in unit tests.
- The result of the first phase of lazy loader process can be serialized out for the purpose of
pretty-printing (akin to `cargo fmt`) of loaded modules.
- The lazy loader itself can be split into separate phases, with the first phase performing superficial
parsing, while a latter phase resolves simple inter-module dependencies (example: a const definition
in module a can simply refer to a const definition in a module b that can be resolved w/o denial-of-service
risks). A possible enhancement at early loader stage could be static type checking (off JSDoc comments),
but we don't aim that in MVP just yet.
- Results of complete lazy loader process are passed to fsengine for execution. Examples of constructs
that should not be processed by the lazy loader are calls to host's intrinsics and to user-defined
functions. Note that it's worth considering execution by "load-time" expressions at a late phase of the loading process since there is no IO, no denial-of-service risks involved.
