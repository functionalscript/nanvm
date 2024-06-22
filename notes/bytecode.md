This note discusses ideas for initial NaNVM’s bytecode specification and bytecode interpreter design.

### Basics

We aim for a simple initial implementation that is parametrized by a set of options (that we name
interpreter’s parameters now on). It’s interesting to run a set of benchmarks varying these parameters
– to get insights on promising areas of improvements (to increase speed and then to reduce memory
footprint – without sacrificing speed).

Our bytecode is stack-based (as opposite to register-based).

“Bytecode” instructions are placed in one continuous array. The interpreter maintains an instruction
pointer (IP) index in the bytecode array. “Call”, "return" and “jump” instructions operate on the IP
value explicitly, while after execution of other instructions the interpreter moves the IP to
proceed to the next instruction implicitly.

The bytecode array is not necessarily an array of bytes. We might get performance benefits from
using a 2-byte or 4-byte or 8-byte instruction word.

One option is to use the 8-byte instruction word model because some instructions might contain
pointers. In that model an instruction that does not contain a pointer has an extra in-instruction
space that might be used for optimizations (more on that below).

Another option is - let's avoid storing pointers in bytecode instructions since that makes bytecode
less persistent. Consider a model when a NaNVM engine loads a compound to evaluate - starting from
a root object. Upon loading, the engine pushes that one and only root object pointer to the stack,
then executes immediate bytecode off that initial VM state (akin to execution of “main”). In this
scheme, the bytecode does not need to have any pointers in it: the “main” function analogue can rely
on the root object at the top of the stack, and any pointers used in the process are produced
dynamically (stored on the stack) – and thus should not be in the bytecode. One benefit on that
pointer-less bytecode strategy is - in absence of pointers, a hash of a compiled bytecode snippet is
akin to a hash of any other content-addressable data.

If we proceed with this pointer-less strategy, we should stick with a variable-length bytecode
scheme most likely. In that scheme the VM starts interpreting an instruction from its first byte,
deciding on what of following bytes belong to the same instruction (moving the IP accordingly). That
scheme yields a more compact and platform-independent bytecode (compared with longer-word fixed-size
instruction schemes).

There are two different kinds of call instructions: bytecode function calls and native function calls.
Native functions, in turn, are either interpreter’s “intrinsics” or foreign functions implemented
outside of the interpreter (they are provided by interpreter’s host). For the sake of simplicity, the
initial interpreter implementation uses intrinsic calls pervasively, even for arithmetic operations.

### On interpreter's stack(s)

It makes sense to implement interpreter’s data stack (to store function call parameters, local
variables, returned values) as an indexed collection of Any-s (let’s name an element of that
collection “a cell”). That data stack grows when the interpreter creates cells for local variables
and function call parameters. It shrinks on returns from functions (since the only data stored on
the top of the stack at this moment is the result; parameter and local variable cells are disposed).

Besides data we need to store function call return IPs – to continue execution of a caller
upon a return. Logically we consider the sequence of current in-flight callees a “callstack”,
but there are options on how to store those return IPs.

One option here is to place each return IP to the data stack (e.g. as an integer number Any – in case
when all bytecode is stored in a Vec). Another option is: let’s store return IPs in a devoted stack
(the actual callstack). Right now that second option looks more attractive (but we might want to
compare both via benchmarks). Nevertheless having the first option in mind let’s consider the
callstack data structure (that is the data stack as well in case of the first option). One of
interpreter’s parameter specifies the maximum depth of the callstack– to stop execution in case of a
bad “infinite recursion” stack overflow situation; that number corresponds to maximum number of
in-flight function calls.

The data stack does not need to have a max size limit in case if we agree on that the standard
out-of-memory panic is OK when the data stack grows too large (which is an extremely rare situation,
given that we always limit callstack's max depth).

### On function arguments and results

Since some of Any-s in the data stack correspond to compound objects (JS arrays, objects) allocated
aside, it might make sense to allocate a JS array of parameters each time a bytecode call is executed.
That array is placed as the Any cell on the top of the data stack; the callee bytecode access that
top cell to read parameters. That array corresponds to JS’s ‘arguments’ array; so, we create a
single-element array when one argument is passed – because the callee might refer to that single
argument not by its parameter name, but as ‘arguments[0]’. When no arguments are passed, any
indexing of ‘arguments’ produces an ‘undefined’ value, yet still we need to create a zero-length
parameter array in this case too, allowing the callee to access ‘arguments’ functions like ‘length’.

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
intrinsic calls – from day one. The cost to pay is – all references of ‘arguments’ in a JS function
compile to special bytecode instructions that are hard-coded in the interpreter (as intrinsics,
most likely). This looks like a reasonable price to pay.

We plan to benchmark an optimization of the intrinsic call instruction that capitalizes on unused
bytes of instruction's 8-byte word (if we choose 8-byte instruction scheme and not a variable-length
instruction scheme) - using a more complicated calling convention (that is applicable
to intrinsics only). Since the interpreter implements both sides of an intrinsic call, it can use
these bytes for
- a constant parameter of an operation intrinsic - as in “add 42” where 42 is placed in the
call-intrinsic-add instruction, while the other argument is at the top of the stack;
- indirect references of local variables as in “add C2 C4” that sums the second-from-top cell with
the fourth-from-top cell;
- an indirect reference of a local variable where the result will be stored (instead of storing it at
the top of the stack) as in “add -> C3”;
- a combination of options listed above as in “add C2 42 -> C3”. In that encoding, use of the standard
calling convention could be expressed as “add C0 C0 -> C0”, meaning: “pop one argument from the stack,
pop another argument from the stack, push the result to the stack”.

### Intrinsic and IntrinsicContext traits

TBD
