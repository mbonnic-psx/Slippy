"""The Rust backend: one package per service, one Cargo workspace above them, and the lock at its root."""
from __future__ import annotations

import re

from ...assets import LANGUAGE_ROOT, asset_tree
from ...backends import APP
from ...selection import Selection
from ...services import App
from ...tooling import service_qualifier
from ..backing_services import backing_service_service_files
from ..composition import wire_store
from ..flag_route import wire_entry
from ..flags import flag_reader
from .cargo import lock, with_dependencies

# The toolchain every Rust service in a project builds with, pinned in `rust-toolchain.toml` at the root so
# rustup installs exactly this on a laptop, in a container and in CI, with the components the gate runs.
RUST_TOOLCHAIN = "1.98.1"
# The coverage gate `make test` holds a Rust service to. The skeleton measures 100% — there is one function
# and one test — so 70 is the floor a first slice can spend down to, the same number Go's gate starts at and on
# the Makefile line for the same reason: a project raises it there as the suite earns it.
RUST_COVERAGE_MINIMUM = 70
# Left out of that count, as Go's gate leaves them out: code only `make test-integration` reaches — the Postgres
# adapters, whose tests need a database — and entry points under `src/bin/`, which are wiring. Counted, they
# would read as untested in the one run that deliberately does not test them.
RUST_COVERAGE_IGNORED = r"(_postgres\.rs|/src/bin/)"
RUST_TEST = (
    f"cd {APP} && cargo llvm-cov --locked --fail-under-lines {RUST_COVERAGE_MINIMUM} "
    f"--ignore-filename-regex '{RUST_COVERAGE_IGNORED}'"
)
# The two lint passes: the formatter's check, then clippy with every warning an error, over tests and
# examples too (`--all-targets`) so the code that exercises a module is held to the standard of the module.
RUST_FMT = f"cd {APP} && cargo fmt --check"
RUST_CLIPPY = f"cd {APP} && cargo clippy --locked --all-targets -- -D warnings"
# The template's package name, in the two spellings Rust gives one crate: `delivery-starter` in a manifest,
# `delivery_starter` wherever code names it.
TEMPLATE = "delivery-starter"
# Where `src/lib.rs` lists its modules. The skeleton declares only `health`; which others exist is decided by
# the selection and the target, so the list is computed from the files actually present.
MODULES = "__MODULES__"
TOP_LEVEL_MODULE = re.compile(r"^src/(\w+)(?:\.rs|/mod\.rs)$")


def service_files(event: bool, selection: Selection, target: str = "none") -> dict[str, str]:
    """What this backend puts in a service's directory, keyed relative to it."""
    files = asset_tree(LANGUAGE_ROOT / "rust/app")
    files["Cargo.toml"] = with_dependencies(files["Cargo.toml"], selection)
    if event and not selection.has("memory"):
        # Only for a backend whose event-store axis is not offered yet: a port with a shape and no adapter
        # behind it. Once the axis is asked, the port and its adapters arrive together.
        files.update(asset_tree(LANGUAGE_ROOT / "rust/event-port"))
    files.update(backing_service_service_files(selection, "rust"))
    # The flag reader, only where there is somewhere to deploy (`flags.py`).
    files.update(flag_reader(target, "rust"))
    wire_entry(files, target)
    wire_store(files, selection, "rust")
    files["src/lib.rs"] = declare_modules(files)
    return files


def declare_modules(files: dict[str, str]) -> str:
    """`src/lib.rs` with a `pub mod` line for every top-level module this service was generated with.

    Rust compiles only what a crate declares, so a module file nobody declared is dead text rather than an
    error — which would let a flag reader or an event port arrive and be silently ignored. Computing the list
    from the files present means whatever the selection added is declared, and nothing else is.
    """
    found = sorted(
        {match.group(1) for path in files if (match := TOP_LEVEL_MODULE.match(path))} - {"lib", "main"}
    )
    # The contract suites are what every adapter's tests run, and nothing else: compiled for tests only.
    lines = "".join(
        f"#[cfg(test)]\npub mod {module};\n" if module.endswith("_contract") else f"pub mod {module};\n"
        for module in found
    )
    return files["src/lib.rs"].replace(f"{MODULES}\n", lines)


def crate_name(project_name: str, service: App) -> str:
    """This service's package name: the project's for the first service, `<project>-<service>` after.

    A package name may not start with a digit, which a project name may; such a name is prefixed so the
    workspace still builds.
    """
    name = service_qualifier(project_name, service).lower()
    return f"app-{name}" if name[:1].isdigit() else name


def name_service(project_name: str, service: App, files: dict[str, str]) -> dict[str, str]:
    """One service's files under this project's own package name, in both of Rust's spellings of it."""
    name = crate_name(project_name, service)
    for path in list(files):
        if path.startswith(f"{service.path}/") and path.endswith((".rs", "Cargo.toml")):
            files[path] = files[path].replace(TEMPLATE, name).replace(TEMPLATE.replace("-", "_"), name.replace("-", "_"))
    return files


def repository_files(
    project_name: str, files: dict[str, str], services: list[App], verify: str
) -> dict[str, str]:
    """The workspace above the services, the toolchain pin, the lock, and the verify script.

    One Cargo workspace for the repository, with every service a member: that is Cargo's own answer to a
    repository with more than one package, and it keeps one `Cargo.lock` and one `target/` for all of them. A
    second service is a second `members` entry, not a second dependency set to lock — and a shared crate under
    `packages/` is one more member that services depend on by path.
    """
    members = "".join(f'    "{service.path}",\n' for service in services)
    files["Cargo.toml"] = (LANGUAGE_ROOT / "rust/repository/Cargo.toml").read_text().replace("__MEMBERS__\n", members)
    files["rust-toolchain.toml"] = (
        (LANGUAGE_ROOT / "rust/repository/rust-toolchain.toml").read_text().replace("__TOOLCHAIN__", RUST_TOOLCHAIN)
    )
    files[".cargo/mutants.toml"] = (LANGUAGE_ROOT / "rust/repository/.cargo/mutants.toml").read_text()
    files["Cargo.lock"] = lock([(crate_name(project_name, service), service.selection) for service in services])
    apps = " ".join(service.path for service in services)
    files[verify] = f"""#!/bin/sh
set -eu
for app in {apps}; do
  (cd "$app" && cargo fmt --check && cargo clippy --locked --all-targets -- -D warnings \\
    && cargo llvm-cov --locked --fail-under-lines {RUST_COVERAGE_MINIMUM} --ignore-filename-regex '{RUST_COVERAGE_IGNORED}')
done
"""
    return files
