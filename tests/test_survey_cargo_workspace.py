"""The survey's Cargo workspaces: a workspace root is proposed once, at its root, with commands that cover every
member, and none per member; a workspace below another is a candidate of its own. Entered at the survey's boundary
over trees written on disk, as `test_survey_cargo.py` is."""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from test_survey import write

from slipwai.survey import buildable


class NpmWorkspaceRegressionTest(unittest.TestCase):
    def test_an_npm_workspaces_package_below_an_npm_workspace_root_is_still_owned(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = write(Path(directory), {
                "package.json": json.dumps({"name": "top", "workspaces": ["packages/*"]}),
                "packages/app/package.json": json.dumps({"name": "app", "workspaces": ["inner/*"]}),
            })
            self.assertEqual([r.path for r in buildable(root)], ["."], "the nested npm workspace is owned")
