#!/usr/bin/env python3
import contextlib
import hashlib
import importlib.util
import io
import json
import subprocess
import tempfile
import types
import unittest
from pathlib import Path


PUBLIC_ROOT = Path(__file__).resolve().parents[1]
SCRIPT = PUBLIC_ROOT / "scripts" / "verify-beta-release-contract.py"


class ReleaseVerifierTest(unittest.TestCase):
    def test_accepts_clean_candidate_without_local_git_when_inputs_match(self) -> None:
        module = self._load_module()
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            self._write_candidate(root, module)
            code, report, stderr = self._run(module, self._args(module, root))

            self.assertEqual(code, 0, stderr)
            self.assertTrue(report["verified"])
            self.assertEqual(report["failure_code"], None)
            self.assertEqual(report["local_checkout_line"], "none")
            self.assertEqual(report["first_success_root_binding"], "candidate_root")

    def test_rejects_mutable_latest_source(self) -> None:
        module = self._load_module()
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            self._write_candidate(root, module)
            args = self._args(module, root)
            args[args.index("--source-sha") + 1] = "latest"
            code, report, stderr = self._run(module, args)

            self.assertNotEqual(code, 0)
            self.assertEqual(report["failure_code"], "BETA-MUTABLE-INPUT")
            self.assertEqual(stderr.strip(), "BETA-MUTABLE-INPUT")

    def test_rejects_absent_ci_run_id_argument(self) -> None:
        module = self._load_module()
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            self._write_candidate(root, module)
            args = self._args(module, root)
            del args[args.index("--ci-run-id") : args.index("--ci-run-id") + 2]
            code, report, _ = self._run(module, args)

            self.assertNotEqual(code, 0)
            self.assertEqual(report["failure_code"], "BETA-MISSING-ARG")

    def test_rejects_wrong_tag_target(self) -> None:
        module = self._load_module()
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            self._write_candidate(root, module)
            args = self._args(module, root)
            args[args.index("--tag-target") + 1] = "0" * 40
            code, report, _ = self._run(module, args)

            self.assertNotEqual(code, 0)
            self.assertEqual(report["failure_code"], "BETA-TAG-TARGET-MISMATCH")

    def test_rejects_wrong_asset_digest(self) -> None:
        module = self._load_module()
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            self._write_candidate(root, module)
            args = self._args(module, root)
            args[args.index("--asset-digest") + 1] = "sha256:" + ("0" * 64)
            code, report, _ = self._run(module, args)

            self.assertNotEqual(code, 0)
            self.assertEqual(report["failure_code"], "BETA-ASSET-DIGEST-MISMATCH")

    def test_rejects_dirty_local_checkout_before_remote_evidence(self) -> None:
        module = self._load_module()
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            self._write_candidate(root, module)
            self._init_git(root)
            (root / "README.md").write_text("dirty\n", encoding="utf-8")
            code, report, _ = self._run(module, self._args(module, root))

            self.assertNotEqual(code, 0)
            self.assertEqual(report["failure_code"], "BETA-DIRTY-LOCAL-CHECKOUT")

    def test_rejects_stale_local_checkout_before_remote_evidence(self) -> None:
        module = self._load_module()
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            self._write_candidate(root, module)
            self._init_git(root)
            code, report, _ = self._run(module, self._args(module, root))

            self.assertNotEqual(code, 0)
            self.assertEqual(report["failure_code"], "BETA-STALE-LOCAL-CHECKOUT")

    def test_rejects_copied_first_success_report_from_other_root(self) -> None:
        module = self._load_module()
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            self._write_candidate(root, module)
            report_path = root / module.FIRST_SUCCESS_REPORT
            report = json.loads(report_path.read_text(encoding="utf-8"))
            report["state_root"] = str(root / "other-public-run")
            report_path.write_text(json.dumps(report, indent=2, sort_keys=True), encoding="utf-8")
            code, report, _ = self._run(module, self._args(module, root))

            self.assertNotEqual(code, 0)
            self.assertEqual(report["failure_code"], "BETA-FIRST-SUCCESS-ROOT-MISMATCH")

    def _load_module(self) -> types.ModuleType:
        spec = importlib.util.spec_from_file_location("verify_beta_release_contract", SCRIPT)
        self.assertIsNotNone(spec)
        self.assertIsNotNone(spec.loader)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        module.github_json = self._github_json(module)
        return module

    def _github_json(self, module):
        def fake(path: str) -> dict:
            if path == "git/ref/heads/main":
                return {"object": {"sha": module.SOURCE_SHA}}
            if path == f"git/ref/tags/{module.TAG}":
                return {"object": {"sha": module.TAG_TARGET}}
            if path == f"releases/tags/{module.TAG}":
                return {
                    "id": module.RELEASE_ID,
                    "tag_name": module.TAG,
                    "target_commitish": module.TAG_TARGET,
                    "prerelease": True,
                    "assets": [
                        {
                            "id": module.ASSET_ID,
                            "name": module.ASSET_NAME,
                            "size": module.ASSET_SIZE,
                            "digest": module.ASSET_DIGEST,
                        }
                    ],
                }
            if path.startswith("actions/runs/"):
                return {
                    "name": "public-ci",
                    "head_sha": module.SOURCE_SHA,
                    "status": "completed",
                    "conclusion": "success",
                }
            raise AssertionError(f"unexpected GitHub path: {path}")

        return fake

    def _run(self, module, args: list[str]) -> tuple[int, dict, str]:
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            code = module.main(args)
        return code, json.loads(stdout.getvalue()), stderr.getvalue()

    def _args(self, module, root: Path) -> list[str]:
        return [
            "--candidate-root",
            str(root),
            "--source-sha",
            module.SOURCE_SHA,
            "--tag-target",
            module.TAG_TARGET,
            "--release-id",
            str(module.RELEASE_ID),
            "--asset-id",
            str(module.ASSET_ID),
            "--asset-digest",
            module.ASSET_DIGEST,
            "--ci-run-id",
            "24947529643",
            "--cgr-output-target",
            module.CGR_OUTPUT_TARGET,
            "--first-success-report",
            module.FIRST_SUCCESS_REPORT,
        ]

    def _write_candidate(self, root: Path, module) -> None:
        root = root.resolve()
        for relative in [
            "README.md",
            "docs/BETA_CRITERIA.md",
            "scripts/first-success.sh",
            "scripts/check-beta-release-contract.sh",
            ".github/workflows/public-ci.yml",
            "Cargo.toml",
        ]:
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("fixture\n", encoding="utf-8")

        state_root = (root / "target" / "public-run").resolve(strict=False)
        cyrune_home = (state_root / "home").resolve(strict=False)
        evidence_dir = cyrune_home / "ledger" / "evidence" / "EVID-1"
        terminal_dir = cyrune_home / "ledger" / "terminal-bindings"
        working_dir = cyrune_home / "working"
        evidence_dir.mkdir(parents=True)
        terminal_dir.mkdir(parents=True)
        working_dir.mkdir(parents=True)

        working_hash = self._write_json(working_dir / "working.json", {"slots": []})
        manifest_hash = self._write_json(
            evidence_dir / "manifest.json",
            {
                "outcome": "accepted",
                "evidence_id": "EVID-1",
                "correlation_id": "RUN-20260501-0801",
                "run_id": "RUN-20260501-0801-R01",
                "policy_pack_id": module.POLICY_PACK_ID,
                "citation_bundle_id": "CB-20260501-0801",
                "working_hash_after": working_hash,
                "rr_present": True,
            },
        )
        hashes_hash = self._write_json(evidence_dir / "hashes.json", {"files": {}})
        self._write_json(
            terminal_dir / "EVID-1.json",
            {
                "schema_version": "cyrune.free.terminal-binding.v1",
                "outcome": "accepted",
                "evidence_id": "EVID-1",
                "working_json_hash": working_hash,
                "evidence_manifest_hash": manifest_hash,
                "evidence_hashes_hash": hashes_hash,
            },
        )
        self._write_json(
            root / module.FIRST_SUCCESS_REPORT,
            {
                "schema_version": module.C5_SCHEMA_VERSION,
                "verified": True,
                "outcome": "accepted",
                "failure_code": None,
                "failure_message": None,
                "public_first_success_input": module.PUBLIC_FIRST_SUCCESS_INPUT,
                "run_mode": module.RUN_MODE,
                "state_root": str(state_root),
                "cyrune_home": str(cyrune_home),
                "response_to": "REQ-20260501-0801",
                "correlation_id": "RUN-20260501-0801",
                "run_id": "RUN-20260501-0801-R01",
                "evidence_id": "EVID-1",
                "policy_pack_id": module.POLICY_PACK_ID,
                "citation_bundle_id": "CB-20260501-0801",
                "working_hash_after": working_hash,
                "working_json_hash": working_hash,
                "evidence_dir": "ledger/evidence/EVID-1",
                "terminal_binding_path": "ledger/terminal-bindings/EVID-1.json",
                "evidence_manifest_hash": manifest_hash,
                "evidence_hashes_hash": hashes_hash,
                "response": {
                    "outcome": "accepted",
                    "correlation_id": "RUN-20260501-0801",
                    "run_id": "RUN-20260501-0801-R01",
                    "evidence_id": "EVID-1",
                    "policy_pack_id": module.POLICY_PACK_ID,
                    "citation_bundle_id": "CB-20260501-0801",
                    "working_hash_after": working_hash,
                },
            },
        )

    def _write_json(self, path: Path, value: dict) -> str:
        path.parent.mkdir(parents=True, exist_ok=True)
        data = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
        path.write_bytes(data)
        return "sha256:" + hashlib.sha256(data).hexdigest()

    def _init_git(self, root: Path) -> None:
        subprocess.run(["git", "init"], cwd=root, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        subprocess.run(
            ["git", "config", "user.email", "test@example.invalid"],
            cwd=root,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        subprocess.run(
            ["git", "config", "user.name", "CYRUNE Test"],
            cwd=root,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        subprocess.run(["git", "add", "."], cwd=root, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        subprocess.run(
            ["git", "commit", "-m", "fixture"],
            cwd=root,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )


if __name__ == "__main__":
    unittest.main()
