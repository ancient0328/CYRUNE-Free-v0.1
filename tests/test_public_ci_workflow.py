#!/usr/bin/env python3
import unittest
from pathlib import Path


PUBLIC_ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = PUBLIC_ROOT / ".github" / "workflows" / "public-ci.yml"
REPORT_PATH = "target/public-run/first-success-report.json"


class PublicCiWorkflowTest(unittest.TestCase):
    def setUp(self) -> None:
        self.workflow = WORKFLOW.read_text(encoding="utf-8")

    def test_semantic_first_success_verifier_step_is_present(self) -> None:
        self.assertIn("- name: Check public first-success semantic verifier", self.workflow)
        self.assertIn("id: first_success_verifier", self.workflow)
        self.assertIn("bash scripts/doctor.sh", self.workflow)
        self.assertIn("bash scripts/first-success.sh", self.workflow)

    def test_first_success_report_artifact_path_is_preserved(self) -> None:
        self.assertIn("- name: Upload public first-success report", self.workflow)
        self.assertIn("if: always()", self.workflow)
        self.assertIn("uses: actions/upload-artifact@v4", self.workflow)
        self.assertIn("name: public-first-success-report", self.workflow)
        self.assertIn(f"path: {REPORT_PATH}", self.workflow)
        self.assertIn("if-no-files-found: ignore", self.workflow)
        self.assertIn("retention-days: 30", self.workflow)

    def test_static_public_envelope_checker_is_not_full_release_proof(self) -> None:
        static_step = self._step("- name: Check public envelope predicates")
        prepared_static_step = self._step("- name: Check prepared public envelope predicates")

        self.assertIn("run: bash scripts/check-public-envelope.sh", static_step)
        self.assertIn("run: bash scripts/check-public-envelope.sh", prepared_static_step)
        self.assertNotIn("check-beta-release-contract.sh", static_step)
        self.assertNotIn("check-beta-release-contract.sh", prepared_static_step)

    def test_release_contract_step_uses_explicit_evidence_inputs(self) -> None:
        release_step = self._step("- name: Check beta release contract")

        self.assertIn("if: github.event_name == 'push'", release_step)
        self.assertIn('bash scripts/check-beta-release-contract.sh \\', release_step)
        self.assertIn('--candidate-root "$PWD" \\', release_step)
        self.assertIn('--source-sha "$BETA_SOURCE_SHA" \\', release_step)
        self.assertIn('--tag-target "$BETA_TAG_TARGET" \\', release_step)
        self.assertIn('--release-id "$BETA_RELEASE_ID" \\', release_step)
        self.assertIn('--asset-id "$BETA_ASSET_ID" \\', release_step)
        self.assertIn('--asset-digest "$BETA_ASSET_DIGEST" \\', release_step)
        self.assertIn('--ci-run-id "$BETA_CI_RUN_ID" \\', release_step)
        self.assertIn(f'--first-success-report "{REPORT_PATH}"', release_step)

    def test_rust_tests_use_prepared_shipping_home(self) -> None:
        rust_test_step = self._step("- name: Check Rust tests")

        self.assertIn(
            "CYRUNE_TEST_SHIPPING_HOME_ROOT: ${{ github.workspace }}/target/public-run/home",
            rust_test_step,
        )
        self.assertIn(
            "run: cargo test --manifest-path Cargo.toml --workspace --all-targets",
            rust_test_step,
        )

    def _step(self, marker: str) -> str:
        start = self.workflow.index(marker)
        next_step = self.workflow.find("\n      - name:", start + len(marker))
        if next_step == -1:
            return self.workflow[start:]
        return self.workflow[start:next_step]


if __name__ == "__main__":
    unittest.main()
