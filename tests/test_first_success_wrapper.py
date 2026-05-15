#!/usr/bin/env python3
import os
import shutil
import stat
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


PUBLIC_ROOT = Path(__file__).resolve().parents[1]
SCRIPT = PUBLIC_ROOT / "scripts" / "first-success.sh"


class FirstSuccessWrapperTest(unittest.TestCase):
    def test_success_writes_verifier_report_to_public_run_state_root(self) -> None:
        with self._candidate_root(
            textwrap.dedent(
                """\
                #!/usr/bin/env bash
                set -euo pipefail
                mkdir -p "$CYRUNE_HOME"
                printf '%s\\n' "$CYRUNE_HOME" > "$CYRUNE_HOME/../observed-home.txt"
                if [ "$*" != "verify first-success" ]; then
                  exit 9
                fi
                printf '%s\\n' '{"verified":true,"outcome":"accepted"}'
                """
            )
        ) as root_name:
            root = Path(root_name)
            result = subprocess.run(
                ["bash", str(root / "scripts" / "first-success.sh")],
                check=False,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

            report_path = root / "target" / "public-run" / "first-success-report.json"
            observed_home = root / "target" / "public-run" / "observed-home.txt"
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(report_path.read_text(encoding="utf-8").strip(), result.stdout.strip())
            self.assertEqual(
                observed_home.read_text(encoding="utf-8").strip(),
                str(root / "target" / "public-run" / "home"),
            )

    def test_verifier_failure_propagates_non_zero_status(self) -> None:
        with self._candidate_root(
            textwrap.dedent(
                """\
                #!/usr/bin/env bash
                set -euo pipefail
                printf '%s\\n' '{"verified":false,"outcome":"rejected","failure_code":"FSV-TEST"}'
                exit 42
                """
            )
        ) as root_name:
            root = Path(root_name)
            result = subprocess.run(
                ["bash", str(root / "scripts" / "first-success.sh")],
                check=False,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

            report_path = root / "target" / "public-run" / "first-success-report.json"
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(report_path.read_text(encoding="utf-8").strip(), result.stdout.strip())
            self.assertIn('"failure_code":"FSV-TEST"', report_path.read_text(encoding="utf-8"))

    def test_wrapper_invokes_semantic_verifier_not_raw_run(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")

        self.assertIn('"$STATE_ROOT/bin/cyr" verify first-success', script)
        self.assertNotIn(" cyr run ", script)
        self.assertNotIn('"$STATE_ROOT/bin/cyr" run', script)
        self.assertIn('REPORT_PATH="$STATE_ROOT/first-success-report.json"', script)

    def _candidate_root(self, fake_cyr_body: str):
        temp = tempfile.TemporaryDirectory()
        root = Path(temp.name)
        scripts_dir = root / "scripts"
        cyr_dir = root / "target" / "public-run" / "bin"
        scripts_dir.mkdir(parents=True)
        cyr_dir.mkdir(parents=True)
        shutil.copy2(SCRIPT, scripts_dir / "first-success.sh")
        fake_cyr = cyr_dir / "cyr"
        fake_cyr.write_text(fake_cyr_body, encoding="utf-8")
        fake_cyr.chmod(fake_cyr.stat().st_mode | stat.S_IXUSR)
        return temp


if __name__ == "__main__":
    unittest.main()
