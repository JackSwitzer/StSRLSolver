This directory contains the pre-rebuild training stack.

It is kept only as historical reference while the new combat-first training
stack is built in `packages/training` on the stacked training branch.

The legacy stack is intentionally out of the default implementation path for:

- new training development
- new benchmarks
- new inference/model interfaces
- new scripts and workflow entrypoints

If a capability from this directory is still useful, port it intentionally into
the new stack instead of extending the legacy code in place.
