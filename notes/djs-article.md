## Addressing JSON pain points via Data JS
JSON is the de-facto lingua franca of data exchange formats, yet, it has well-known shortcomings - repeatedly addressed in a number of projects like [JSON5](https://json5.org/) and [Hjson](https://github.com/hjson/hjson-js). Here we consider a promising new approach in that area.

Let's take a random motivating example of a "JSON+" data format: a [JSON Template Engine](https://github.com/vmware-archive/json-template-engine)) of a... what entity, could you guess by the name of the project? Could you guess what a dollar sign or a hash sign would mean in that generically-sounding "JSON Template engine" data format? Projects like that are numerous. It's easy, actually, to create a custom DSL extension on top a vanilla JSON - and - live happily ever after (or, more frequently, abandon that project - that disappears, naturally, into obscurity).

In this article we lay down a vendor-agnostic approach for extending JSON with long-time wished-for features expressed in a historically well-known syntax that is familiar and thus natural to human users. We name it Data JS. Please let me explain how exactly Data JS tries to avoid common pitfalls of prior JSON extensions - the key point here is: Data JS does not introduce any new language at all.

Let's consider what JSON pain points do we want to address with Data JS, and - by what means (spoiler: there will be no special meanings of a dollar sign, or a hash sign, or of any other funny symbol!).

What is the point of view in focus of this consideration, "who are 'we'?" in here? From a point of view of my prior "generalist" programmer's experience, I'll elaborate on that below. My personal background involves decades of a system-level C/C++ programming, with quite a dash of JS / TypeScript on top of that.
