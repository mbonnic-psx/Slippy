PATCH

**`slipwai adopt` no longer writes a directory's name into a shell command or the Makefile when the shell or
make would read part of it as code.** The survey put each buildable directory's path into `cd <dir> && …` and
into `delivery/Makefile` as it was. A directory named with a newline made `$(shell …)` top-level make, run
while make parsed, so even `make help` executed it; a `;`, `$(…)` or quote reached `sh -c` on `make verify`. A
directory whose name holds anything but letters, digits and `._+-`, or starts with `-`, is no longer surveyed
(experimental, with the rest of adoption). The report names it, escaped, as the quick win `unsafe-path`, with
the fix: rename it. `cd` also quotes what it is given.

**Catch-up.** A repository adopted before this may already record such a path. Every command that reads
`project.json` now refuses it and names the deployable, `adopt --refresh` included. Rename the directory and
record the new path, or remove the deployable. Either way, look at who committed that name first: until then,
don't run `make` in that checkout.
