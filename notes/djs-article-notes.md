## Data JS - JSON based "data + code" format
(fix up the name of the article)
### Intro: what are most important messages of the article?
1. JSON is the de-facto lingua franca of data exchange, yet, it has well-known shortcomings - addressed in a number of projects. Here we consider a promising new approach in that area.
2. On concepts: we are programmers. Considering that data is akin to code, we strive for ways to unify our mental models ("code is data and data is code"). While we modularize and de-duplicate code we deal with, we want to have similar experiences with (JSON) data. What are major pain points?
3. Here we list a brief table of content for the reminder of the article (compiled off titles / content of following sections).
### Avoiding redundancy
Explain the cost of unmitigated redundancy.
Data as a DAG of objects, not a tree of objects.
Const definitions / usage.
Modules. Mention both .cjs and .mjs.
Plus, a brief one-paragraph list of commonly-used JSON relaxations / enhancements (JSON5 etc.) that make JSON more human-friendly. What we suggest with JS Data is to introduce a bit more programmer-friendly data+code language.
### On embedding code in data - JS as the best JSON friend
Show a couple of examples of niche ad-hoc DSL-s embedded in JSON values (e.g. Azure templates; JSON Schema).
Embracing a well-controlled subset of JS in Data JS is a straight continuation of the const / modules line discussed above.
Do we take function declarations here - but why not? It's a question of gradual staging of Data JS. That aspect will be covered in future articles.
### Analogy to compile-time calculations
Data JS's MVP is a build-time preprocessing system that allows to
1. generate vanilla JSON out of well-structured .djs source code;
2. provide a playground for mixed well-controlled compile-time / run-time execution environments (bridging to Functional Script);
3. revisit tree-shaking / bundling functionality of JS dev stacks.
### On importance of controlled execution of code in Data JS
Embracing a wide range of JS features in Data JS opens a floodgate of security and performance issues. Thus we don't "step down JS", we step up JSON with well-controlled computation enhancements.
### What do we have as of now?
We have code for Data JS serialization and deserialization that supports modules and const-s, accompanied with popular minor JSON syntax relaxations. Our next MVP goal is - enabling JS expression in values, supporting a limited subset of JS runtime functions.
### What do we plan?
We plan to gradually bridge bottom-to-top Data JS with FunctionalScript.

