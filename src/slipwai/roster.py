"""Growing a generated project's list of applications: the next port of each kind, and the list with one more.

Split from `services.py` when a wrapped application's `runner` took that module past its budget. What an
application is stays there; this is only how `add-service` and `add-frontend` extend the list.
"""
from __future__ import annotations

from collections.abc import Sequence

from .backends import SERVICE_PORT, WEB_PORT
from .errors import GenerationError
from .selection import Selection
from .services import App, check_name, service_app, services_of, web_app, web_apps


def next_port(apps: list[App]) -> int:
    """The next service port: one above the highest a service already has — 3000, 3001, … — so nothing is
    reassigned. The browser apps' 5173, 5174, … are a sequence of their own."""
    return max((app.port for app in services_of(apps)), default=SERVICE_PORT - 1) + 1


def next_web_port(apps: list[App]) -> int:
    return max((app.port for app in web_apps(apps)), default=WEB_PORT - 1) + 1


def add_web(apps: list[App], name: str, api: str | None) -> list[App]:
    """The list with one more browser app on it, proxying `/api` to `api` (the first service by default)."""
    check_name(apps, name)
    services = services_of(apps)
    if not services:
        raise GenerationError(
            "this project has no service the factory made for a browser app to proxy to; add one with add-service first"
        )
    api = services[0].name if api is None else api
    if api not in {service.name for service in services}:
        raise GenerationError(
            f"'{api}' is not a service of this project; the browser app can proxy to "
            f"{', '.join(service.name for service in services)}"
        )
    return [*apps, web_app(name, next_web_port(apps), api)]


def add_service(
    apps: list[App], name: str, backend: str, selection: Selection,
    purpose: str | None = None, contexts: Sequence[str] | None = None,
) -> list[App]:
    """The list with one more service on it, refusing a name that cannot be a service here."""
    check_name(apps, name)
    # The first service the factory makes takes the role (`make dev`, `PORT`) even beside pre-existing ones.
    first = not services_of(apps)
    return [
        *apps,
        service_app(name, backend, next_port(apps), selection, first=first, purpose=purpose, contexts=contexts),
    ]

