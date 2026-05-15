#!/usr/bin/env python3
import re
import unittest
from pathlib import Path


PUBLIC_ROOT = Path(__file__).resolve().parents[1]


def _project_root_with_source_dev_docs(public_root: Path) -> Path | None:
    candidates = [
        public_root.parents[2] / "full" / "v01",
        public_root.parent,
    ]
    for candidate in candidates:
        if (
            candidate
            / "dev-docs"
            / "04-implementation-notes"
            / "CURRENT_BETA_IDENTITY_CANONICAL.md"
        ).is_file():
            return candidate
    return None


PROJECT_ROOT = _project_root_with_source_dev_docs(PUBLIC_ROOT)

CURRENT_BETA_IDENTITY = {
    "source_sha": "062cd58548e9f66e2371f580edae8f641d0d05f7",
    "tag": "v0.1.1-beta.1",
    "tag_target": "61eb4c68630600d9b1a7f325fd6d06759ede846c",
    "asset": "cyrune-free-v0.1.1-beta.1.tar.gz",
}

if PROJECT_ROOT is None:
    DEV_DOCS_CONTRACTS: list[Path] = []
else:
    DEV_DOCS_CONTRACTS = [
        PROJECT_ROOT
        / "dev-docs"
        / "04-implementation-notes"
        / "CURRENT_BETA_IDENTITY_CANONICAL.md",
        PROJECT_ROOT
        / "dev-docs"
        / "04-implementation-notes"
        / "PUBLIC_BETA_TERMINAL_EVIDENCE_CANONICAL.md",
        PROJECT_ROOT
        / "dev-docs"
        / "05-ci"
        / "PUBLIC_BETA_VERIFIER_CI_CANONICAL.md",
    ]

FORBIDDEN_SUCCESS_PATTERNS = [
    re.compile(r"success\s+means\s+.*raw\s+`?cyr\s+run", re.IGNORECASE),
    re.compile(r"first-success\.sh`?\s+(runs|executes)\s+`?cyr\s+run", re.IGNORECASE),
    re.compile(r"cyr\s+run\s+--no-llm\s+--input\s+\"ship-goal public first success\""),
    re.compile(r"exit\s+(code\s+)?0\s+(means|proves)\s+(accepted|success)", re.IGNORECASE),
]


class PublicWordingContractTest(unittest.TestCase):
    def test_current_public_docs_do_not_revert_success_to_raw_cyr_run_or_exit_zero(self) -> None:
        violations: list[str] = []
        for path in self._current_public_markdown_files():
            for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
                if any(pattern.search(line) for pattern in FORBIDDEN_SUCCESS_PATTERNS):
                    violations.append(f"{path.relative_to(PUBLIC_ROOT)}:{lineno}:{line.strip()}")

        self.assertEqual(violations, [])

    def test_first_success_current_wording_binds_to_semantic_verifier(self) -> None:
        required_files = [
            PUBLIC_ROOT / "README.md",
            PUBLIC_ROOT / "README.ja.md",
            PUBLIC_ROOT / "docs" / "FIRST_SUCCESS_EXPECTED.md",
            PUBLIC_ROOT / "docs" / "ENGINEERING_SPEC.md",
            PUBLIC_ROOT / "docs" / "USER_GUIDE.md",
            PUBLIC_ROOT / "docs" / "ja" / "ENGINEERING_SPEC.md",
            PUBLIC_ROOT / "docs" / "ja" / "USER_GUIDE.md",
        ]

        for path in required_files:
            text = path.read_text(encoding="utf-8")
            self.assertIn("cyr verify first-success", text, path)
            self.assertIn("outcome", text, path)
            self.assertIn("accepted", text, path)

    def test_current_beta_identity_tuple_matches_public_and_dev_docs(self) -> None:
        files = [
            PUBLIC_ROOT / "README.md",
            PUBLIC_ROOT / "README.ja.md",
            PUBLIC_ROOT / "docs" / "BETA_CRITERIA.md",
            PUBLIC_ROOT / "docs" / "ENGINEERING_SPEC.md",
            PUBLIC_ROOT / "docs" / "ja" / "BETA_CRITERIA.md",
            PUBLIC_ROOT / "docs" / "ja" / "ENGINEERING_SPEC.md",
        ]
        if DEV_DOCS_CONTRACTS:
            files.extend([DEV_DOCS_CONTRACTS[0], DEV_DOCS_CONTRACTS[2]])

        for path in files:
            text = path.read_text(encoding="utf-8")
            self.assertIn(CURRENT_BETA_IDENTITY["tag"], text, path)
        if DEV_DOCS_CONTRACTS:
            identity_text = DEV_DOCS_CONTRACTS[0].read_text(encoding="utf-8")
            self.assertIn(CURRENT_BETA_IDENTITY["source_sha"], identity_text)
            self.assertIn(CURRENT_BETA_IDENTITY["tag_target"], identity_text)
            self.assertIn(CURRENT_BETA_IDENTITY["asset"], identity_text)

    def test_historical_and_deferred_shelves_are_not_scanned_as_current_truth(self) -> None:
        scanned = {
            path.relative_to(PUBLIC_ROOT).as_posix()
            for path in self._current_public_markdown_files()
            if path.is_relative_to(PUBLIC_ROOT)
        }

        self.assertFalse(any(path.startswith("docs/historical/") for path in scanned))
        self.assertFalse(any(path.startswith("docs/deferred/") for path in scanned))

    def _current_public_markdown_files(self) -> list[Path]:
        files = [PUBLIC_ROOT / "README.md", PUBLIC_ROOT / "README.ja.md"]
        for path in (PUBLIC_ROOT / "docs").rglob("*.md"):
            relative = path.relative_to(PUBLIC_ROOT).as_posix()
            if relative.startswith("docs/historical/") or relative.startswith("docs/deferred/"):
                continue
            files.append(path)
        files.extend(DEV_DOCS_CONTRACTS)
        return sorted(files)


if __name__ == "__main__":
    unittest.main()
