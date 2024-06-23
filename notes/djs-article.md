## Addressing JSON pain points via Data JS
JSON is the de-facto lingua franca of data exchange formats, yet, it has well-known shortcomings - repeatedly addressed in a number of projects like [JSON5](https://json5.org/) and [Hjson](https://github.com/hjson/hjson-js). Here we consider a promising new approach in that area.

Let's take a random motivating example of a "JSON+" data format: a [JSON Template Engine](https://github.com/vmware-archive/json-template-engine)) of a... what entity, could you guess by the name of the project? Could you guess what a dollar sign or a hash sign would mean in that generically-sounding "JSON Template engine" data format? Projects like that are numerous. It's easy, actually, to create a custom DSL extension of a vanilla JSON - and - leave happily ever after (or, more frequently, disappear into obscurity).

In this article we lay down a vendor-agnostic approach for extending JSON with long-time wished-for features expressed in a historically well-known syntax that is familiar and thus natural to human users (that we name "JS" for good reasons - please keep reading!).

We name it Data JS. Let's consider what JSON pain points do we want to address, and in what means (spoiler: there will be no dollar, hash, and and any other funny symbols!).