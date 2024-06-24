## Addressing JSON pain points via Data JS

JSON serves as the universal language for data exchange, but it has known limitations.
These have been tackled by general-purpose extensions like
[JSON5](https://json5.org/) and [Hjson](https://github.com/hjson/hjson-js), as well as
custom extensions in problem-specific systems.

In this article we describe a promising new approach on extending JSON - that avoids
common pitfalls. Let’s delve into a couple of motivating examples of ‘JSON+’ data
formats.

[JSON Template Engine](https://github.com/vmware-archive/json-template-engine/blob/master/templating/README.md):
This tool allows referencing JSON entities within the same or other .json files and
performing limited computations. For instance, given `{"x":[{"y":{"z":1}}]}` earlier in
the data, `"${x[0].y.z}"` evaluates to `"1"` (using `${}` syntax to wrap a JS-like
term). A more complicated example,
`["#for-each", [{"x": 1}, {"x": 2}], "template.json"]`, injects parametrized
content of another .json template (using `#` syntax to specify a predefined processing
function). Despite its relative obscurity, this project provides expressive computation
capabilities for extending JSON in a general-purpose manner.

Our second exemplar JSON extension,
[ARM templates DSL](https://learn.microsoft.com/en-us/azure/azure-resource-manager/templates/syntax),
serves a specific purpose: resource specification in the Microsoft Azure cloud
ecosystem. To enhance this DSL’s expressiveness and flexibility, its creators
introduced an
[expanded set of built-in functions](https://learn.microsoft.com/en-us/azure/azure-resource-manager/templates/template-functions)
and a [limited ability to define user functions](https://learn.microsoft.com/en-us/azure/azure-resource-manager/templates/syntax#functions).

As in the previous example, this DSL allows to reference resource templates defined
in separate .json files. It's syntax for templated JSON values is different (requiring
familiarity with special meanings of  `[]`, `()`, `{}`, `''` within string values, not
`$` and `#` as in the previous example). The JSON extension remains closely tied to its
problem-oriented use cases and isn’t a general-purpose solution.

Numerous custom JSON extensions, like the two we discussed in brief, fulfill their
purposes effectively. However, each extension’s DSL incurs cognitive overhead and
long-term maintenance costs.

In this article, we present Data JS — a vendor-agnostic approach to extending JSON
that does not introduce a DSL language for templated values and cross-file references.
Instead it leverages familiar JavaScript syntax and standard modularization techniques.
Having a general-purpose core, Data JS enables
building problem-oriented applications by bridging to various ‘host environments’.

Data JS addresses several key points:
- It eliminates data redundancy through `const` declarations.
- It enables modular data structuring via cross-file references, using syntax that is
well-familiar to JavaScript users. 
- It allows load-time computations using JavaScript syntax, plus delayed ‘run-time’
computations in a host environment.
- Despite its expressiveness, Data JS maintains security and efficiently manages 
load-time resource consumption (CPU cycles/memory). Our reference Data JS
implementation performs computations within a strict sandboxed environment.

### Deduplication: constants and modules

Code duplication, often referred to as ‘copy and paste programming,’ is an infamous
anti-pattern. To mitigate this, we modularize our programs and factor out repeated code.

To address data duplication (or excessive data ‘copy and paste’), Data JS leverages
JavaScript’s `const` declaration and standard modularization techniques (either
CommonJS `.cjs` or ECMAScript `.mjs` modules). In this article, we use ECMAScript
syntax. Consider the following example from `test.d.mjs` (where the `.d` sub-extension
denotes Data JS content):

```js
import m from "my_module.d.mjs"
const a = "my long string value"
const b = [3, m, a]
// Shape the test data object
export default { foo: [a, b], bar: b }
```

In this snippet, a data entity imported from `my_module.d.mjs` is referred as `m`.
When other data entities are used multiple times, defining them via `const`
declarations eliminates redundancy. The `test.d.mjs` exports exactly one data entity
for external usage. Notably, Data JS implements commonly used JSON relaxations (as
demonstrated in the snippet with comments and non-quoted identifiers as keys).

Data JS employs relative paths in `import` statements, forming a directed acyclic graph
of interconnected modules. Upon loading and processing, the resulting data graph can be
saved in vanilla JSON format (which may be bloated due to loss of deduplication
benefits) or as a bundled singular `.d.mjs` file. In both cases, the output excludes
data that are not referred to from the root data object — akin to the “tree-shaking”
capabilities of JS bundlers.


### Load-time computations

In the snippet above, the use of identifiers (referring to constants or imported
values) represents a simplified form of JavaScript expressions in value contexts. Data
JS supports a well-defined subset of ECMAScript, including standard operators and
functions (referred to as the ‘standard JS prelude’ below).

As we enhance the capabilities of our reference implementation of Data JS, we plan to:
- Allow to extend the standard JS prelude with custom external pre-defined objects and
functions (creating a ‘host environment’).
- Support user-defined functions that Data JS executes in a well-controlled manner.

These enhancements bridge Data JS to
[FunctionalScript](https://medium.com/@sergeyshandar/list/functional-programming-in-javascript-495efca5536a).
Loaded Data JS datasets containing user-defined functions form an in-memory structured
code + data compound that operates on the host environment ‘at run time’ (in contrast
to code executed ‘at load time’).

Our reference implementation of Data JS transforms JavaScript code into bytecode
arrays, some of which execute on the fly ‘at load time.’ However, unrestricted
computations may pose security risks and lead to unbounded CPU and/or memory
consumption. Data JS mitigates this by enforcing adjustable quotas, aborting data
loading sessions that exceed these limits — similar to how a C++ or Rust compiler
panics when compile-time calculations exceed built-in restrictions.

Upon successful completion of a load session, the host system can serialize the
resulting Data JS compound for future reuse. Subsequent re-loading of such a ‘saved
snapshot’ is quicker and requires less memory compared to the initial loading, which
involved JavaScript code compilation and load-time execution. This process is analogous
to the [native-image feature of Graal VM](https://www.graalvm.org/latest/reference-manual/native-image/).

### Current status of Data JS

Our reference implementation of Data JS, written in Rust, includes a partially
functional bytecode interpreter VM called NaNVM. We’ve developed a Data JS loader
(deserializer) and a serializer that both handle const declarations and modularization.
However, our JS-to-bytecode compiler and bytecode interpreter are not yet functional.
In the meantime, we experiment with Data JS snippets containing JS code using external
JS execution engines like Node.js.

As part of NaNVM’s stress-testing and development, we intend to port some of the
Rust-based NaNVM code to FunctionalScript. In this model, the NaNVM Rust core serves as
the host environment, and FunctionalScript NaNVM code calls into the core as necessary.

We intend to share future progress on the NaNVM project and delve into its design
details in upcoming articles.