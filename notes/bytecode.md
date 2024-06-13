This note discusses ideas for initial NaNVM’s bytecode specification and bytecode interpreter design.

### Basics

We aim for a simple initial implementation that is parametrized by a set of options (that we name
interpreter’s parameters now on). It’s interesting to run a set of benchmarks varying these parameters
– to get insights on promising areas of improvements (to increase speed and then to reduce memory
footprint – without sacrificing speed).

“Bytecode” instructions are placed in one continuous array – not necessarily an array of bytes.
We might get performance benefits from using a 2-byte or 4-byte or 8-byte instruction word.

The interpreter maintains an instruction pointer (IP) index in the bytecode array. “Call”, "return"
and “jump” instructions operate on the IP value, while after execution of other instructions the
interpreter increments the IP to proceed to the next instruction.

There are two different kinds of call instructions: bytecode function calls and native function calls.
Native functions, in turn, are either interpreter’s “intrinsics” or foreign functions implemented
outside of the interpreter (they are provided by interpreter’s host). For the sake of simplicity, the
initial interpreter implementation uses intrinsic calls pervasively, even for arithmetic operations.

### On interpreter's stack(s)

Our bytecode is stack-based (as opposite to register-based).

It makes sense to implement interpreter’s data stack (to store function call parameters, local
variables, returned values) as an an indexed collection of Any-s (let’s name an element of that
collection “a cell”). That data stack grows when the interpreter creates cells for local variables
and function call parameters (for the sake of simplicity it makes sense to treat parameters as local
variables).

Besides data we need to store function call return IP values – to continue execution of a caller
after a callee returns. Logically we consider the sequence of current in flight callees a “callstack”,
but there are options on how to store those return IPs.

One option here is to place each return IP to the data stack (e.g. as an integer number Any – in case
when all bytecode is stored in a Vec). Another option is: let’s store return IPs in a devoted stack
(the actual callstack). Right now that second option looks more attractive (but might want to compare
both via benchmarks). Nevertheless having the first option in mind let’s consider the callstack data
structure (that is the data stack as well in case of the first option). One of interpreter’s parameter
specifies the maximum depth of the callstack– to stop execution in case of a bad “infinite recursion”
stack overflow situation; that number corresponds to maximum number of function calls in flight.
The data stack does not need to have a max size limit in case if we agree on that the standard
out-of-memory panic is OK when the data stack grows too large (which is an extremely rare situation,
given that we always limit callstack's max depth).

### On function arguments and results

Since some of Any-s in the data stack correspond to compound objects (JS arrays, objects) allocated
aside, it might make sense to allocate a JS array of parameters each time a bytecode call is executed.
That array is placed as the Any cell on the top of the data stack; the callee bytecode access that
top cell to read parameters. That array corresponds to JS’s ‘arguments’ array; so we create a
single-element array when one argument is passed – because the callee might refer to that single
argument not by its parameter name, but as ‘arguments[0]’. Even when any indexing ‘arguments’ raises
an exception in case when there are no arguments, a zero-length array at the top of the stack is
needed in this model, allowing the callee to access ‘arguments’ functions like ‘length’.

However, since we plan to implement even simple arithmetic operations as intrinsics, it makes sense
to have a more efficient calling convention for intrinsics (expressed via a pair of traits: an
intrinsic trait and an intrinsic context trait – discussed below). The interpreter places intrinsic
parameters sequentially on the data stack, and passes a correspondent slice of the data stack to the
intrinsic (that slice is empty when the intrinsic has no parameters). In turn, the called intrinsic
returns another slice of the data stack with results (that slice is empty in case when the intrinsic
has no results).

After implementing this calling convention for intrinsics – that does not involve memory allocations
aside of the data stack – we might ask ourselves “why not to use that allocation avoidance tactic for
bytecode function calls as well?”

Yes we can implement this optimization – that by the way unifies calling conventions of bytecode and
intrinsic calls – from day one. The cost to pay is – any reference of ‘arguments’ in a JS function
compiles to special bytecode instructions hard-coded in the interpreter (intrinsics, most likely).
This looks like a reasonable price to pay.

### Intrinsic and IntrinsicContext traits

TBD
