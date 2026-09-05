# Exact instruction-strand optimization

For user-authored transformations, [reflection](REFLECTION.md) separately
provides `rewrite(graph, pattern, replacement)` and `apply(graph, inputs...)`.
Unlike the theorem-backed `untrace` path below, an explicit rewrite makes no
equivalence or shortening claim. Its rule and acceptance checks belong in source.

## Requirement

An operation trace is already a native coordinate object. Calling
`untrace(trace(function), 1)` must therefore be able to rebuild a smaller
equivalent instruction graph without converting program output into training
data.

## Invariant

The rank-one result may replace the input strand only when all of these hold:

1. the complete canonical strand decodes, including every dependency and call
   edge;
2. every rewrite is authorized by a theorem listed for the compiler optimizer;
3. the reconstructed strand is canonical and has fewer instruction
   coordinates;
4. no instruction relationship is discarded.

The optimizer returns the original strand when no authorized shorter candidate
exists. Rank below one is rejected because frequency removal alone does not
prove that the reconstructed program remains executable or equivalent.

## Representation

The decoder reads the linked continuation coordinates rather than treating
instructions as an unordered bag. Every coordinate records an instruction
kind, arguments, source fields, and its exact position. Prefix arities rebuild
the expression tree; function boundaries and call names rebuild the finite
function graph. Recursive calls remain finite graph edges.

Zero-valued metadata fields disappear under ordinary native ADD. The decoder
therefore reconstructs zero only where the trace schema defines it as the
unique absent default, such as the first parameter position, a non-variadic
flag, a zero orientation, or camera destination zero.

## Accepted tradeoff

This version uses the existing finite theorem-backed rewrite allowlist. It does
not introduce local storage, common-subexpression caching, loop synthesis, or
lossy instruction selection. Those transformations require additional
language semantics and equivalence rules; representing repeated instruction
frequencies is not by itself sufficient authority to add them.

Expression reconstruction is bounded to 1,024 nested nodes so malformed or
adversarial strands return `NSI004` instead of overflowing the process stack.
