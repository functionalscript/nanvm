## Bridging the Gap from JSON to JavaScript without DSLs

In this article, we explore a natural approach to enhancing JSON by seamlessly integrating essential
features using JavaScript constructs. This approach differs from introducing custom DSL enhancements to JSON.

Apart from addressing human-friendly syntactic enhancements (as tackled by [JSON5](https://json5.org/),
[Hjson](https://github.com/hjson/hjson-js), and similar approaches), we focus on the following pain points:

1. Duplication Avoidance:
    - Traditional JSON’s lack of support for directed acyclic data graphs results in verbose, repetitive
    content. This redundancy can lead to copy/paste errors and cognitive overload - while a simple JS
    “no-DSL” enhancement addresses that major pain point.
2. Modularization:
    - Just as code is modularized, programmers naturally desire to modularize data. Again, we aim to address
    this need with well known standard JS syntax.
3. Efficient Load-Time Computations:
    - Our goal is to enable load-time computations while maintaining robust security measures. Once again,
    there is no custom DSL in our model here: we use standard JS with well-defined restrictions targeting
    security (which includes well-controlled predictable outcome of load-time computations).

In the rest of this article we name the resulting JSON extension “DJS”.

### Exploring Motivating Examples: JSON Extensions via DSLs

Let's consider few JSON extensions varying on the following points:
- Does a given extension maintain JSON purity (hiding its DSL within strings literals)?
- Is it a general-purpose or a problem-oriented extension?
- How expressive is it (especially regarding user-defined functions)?

Motivational examples listed below are chosen rather randomly from a wide field of JSON extensions.

1. [JSON Template Engine](https://github.com/vmware-archive/json-template-engine/blob/master/templating/README.md):
This tool allows referencing JSON entities within the same or other .json files and
performing limited computations. For instance, given `{"x":[{"y":{"z":1}}]}` earlier in
the data, `"${x[0].y.z}"` evaluates to `"1"` (using `${}` syntax in a string literal to wrap
a JS-like term). A more complicated example, `["#for-each", [{"x": 1}, {"x": 2}], "template.json"]`, injects
parameterized content of another JSON template in a loop (using a built-in `for_each` function).
Despite its relative obscurity, this project provides expressive computation capabilities for
extending JSON in a general-purpose manner. It achieves this by using a unique DSL hidden in string
values, thus preserving JSON purity. To master this JSON extension one has to learn
the DSL syntax elements (`${}`, `#`) plus a compact set of built-in functions. There is no
support for user-defined functions.

2. [ARM templates DSL](https://learn.microsoft.com/en-us/azure/azure-resource-manager/templates/syntax)
serves a specific purpose: resource specification in the Microsoft Azure cloud
ecosystem. To enhance this DSL’s expressiveness and flexibility, its creators
introduced an
[expanded set of built-in functions](https://learn.microsoft.com/en-us/azure/azure-resource-manager/templates/template-functions)
and a [limited ability to define user functions](https://learn.microsoft.com/en-us/azure/azure-resource-manager/templates/syntax#functions).
As in the previous example, this DSL allows referencing resource templates defined
in separate JSON files. Its syntax for templated JSON values is different (requiring
familiarity with special meaning of  `[]`, `()`, `{}`, `''` interpolation within string values, not
`${}` and `#` as in the previous example). This JSON extension remains closely tied to its
problem-oriented use cases (serving them very well!) and thus isn’t a general-purpose
solution. It has its custom modularization features and keeps JSON purity (as in the previous
example, its DSL syntax hides in string values, plus, certain key strings have special meaning).


3. [Jsonnet](https://jsonnet.org/) is a well-defined and elaborated configuration language
that, in fact, provides a rich set of general-purposed JSON manipulation functionality.
Unlike the previous examples, Jsonnet diverges from pure JSON syntax. Its extended
JSON-like syntax resembles JavaScript closely but yet is different: for example, it uses its own
`local` (and not JS's `const`) keyword for defining constants. Jsonnet allows multi-file modular
construction of configuration data. Its syntax differs from popular JavaScript modularization
syntaxes. Its evaluate-to-JSON semantic has a formal definition; Jsonnet's user-defined
functionality support is powerful enough to define the “rich” Jsonnet language in a form
of a standard prelude based on a minimalistic core language.

Numerous custom JSON extensions, like the three that we briefly introduced above,
fulfill their purposes effectively. However, each extension’s DSL incurs added complexity
and long-term maintenance costs. DJS’s approach has benefits of reusing familiar
JavaScript syntax and encapsulation techniques - instead of defining yet another DSL.

### DJS’s approach to deduplication: constants and modules

Code duplication, often referred to as ‘copy and paste programming’, is an infamous
anti-pattern. To mitigate it, we modularize our programs and factor out repeated code.

To address data duplication (or excessive data ‘copy and paste’), DJS leverages JavaScript’s
`const` declarations and standard modularization techniques (either CommonJS `.cjs`, or ECMAScript
`.mjs` modules). In this article, we use ECMAScript module syntax. Consider the following example
from `test.d.mjs` (where the `.d` sub-extension denotes DJS content):

```js
import m from "my_module.d.mjs"
const a = "my long string value"
const b = [3, m, a]
// Shape the test data object
export default { foo: [a, b], bar: b }
```

In this snippet, we refer to a data entity imported from `my_module.d.mjs` as `m`.
When other data entities are used multiple times, defining them via `const`
declarations eliminates redundancy. In its last statement `test.d.mjs` exports exactly one data
entity for external usage. Notably, Data JS implements commonly used JSON relaxations (as
demonstrated in the snippet with a comment and a use of non-quoted identifiers as keys).
Extensions listed here are elements of ECMAScript standard (as opposite to yet another DSL).

DJS employs relative paths in `import` statements, forming a directed acyclic graph
of interconnected modules. After loading and processing, you can save the resulting data graph
either in vanilla JSON format (which might become bloated due to the loss of deduplication benefits)
or as a bundled singular `.d.mjs` file. In both cases, the output excludes
data that are not referred to from the root data object — that's akin to the “tree-shaking”
capabilities of JS bundlers.

### “Load-time” and “run-time” computations

In the snippet above, we see the use of identifiers (referring to constants or imported values).
This represents a simple case of using JavaScript expressions in value contexts. DJS will support
a subset of ECMAScript that includes:
- in-place value expressions fully compliant with ECMAScript standard;
- a restricted subset of ECMAScript's standard functions;
- a restricted form of user-defined functions;
- an FFI (foreign function interface) facility for calling host-defined “problem-oriented” native
functions.

Our reference implementation of DJS will support serialization of a loaded DJS graph
with a set of user's code snippets preserved for delayed “run-time” execution (while other
“immediate” DJS code gets executed at load time).

### Current status of DJS

[Our current DJS implementation](https://github.com/functionalscript/nanvm) features modular loading (deserialization) and saving
(serialization) - with support of both CommonJS and ECMAScript syntax for
modules. Constant declarations are supported on both sides (loading / saving),
plus, the loader supports several JavaScript-compatible syntax relaxations of JSON.

Our next steps involve gradually implementing JavaScript expressions and introducing limited
user-defined function definitions.

DJS’s compatibility with both CommonJS and ECMAScript module syntax is crucial during this
interim phase. That detail allows us to experiment with DJS’s future target scenarios using
an external JavaScript facility (Node, Deno, Bun, or similar tools).

In contrast to executing via an external JavaScript engine, our future full-scale
reference DJS implementation will define DJS's restricted subset of ECMAScript standard
(to provide security and resource consumption control guarantees that are hard to achieve when
using an external JavaScript engine).

Our plan is to share ongoing progress on the DJS project and explore its design details in
upcoming articles.
