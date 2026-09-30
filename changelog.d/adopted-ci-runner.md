MINOR

**An adopted application can say what its CI runner needs beyond a toolchain.** A desktop application builds
against system libraries that no `setup-*` action installs, and its smoke opens a window, which a runner cannot
do. Recorded under the application's `runner` in `project.json`, `packages` (apt package names) are installed
before `install` in both of the gate's jobs, and `display: true` runs `make smoke` under `xvfb-run`, with `xvfb`
installed in the smoke job only. A GitLab job names the packages its image must carry, since not every image has
apt. Each name is held to Debian's package-name rule when `project.json` is read, and anything else in `runner`
is refused, because the names reach a shell line on the runner and a pull request can change the record. `/ground`
asks for them where it asks how an application starts.

**Catch-up.** None is needed. A repository whose smoke job is red because the runner lacks a library or a display
records `runner` for that application, then runs `slipwai adopt --refresh`.
