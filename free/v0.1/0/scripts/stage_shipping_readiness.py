#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import json
import os
import platform
import shutil
import stat
import subprocess
import tarfile
import textwrap
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

PRODUCT = "cyrune-free"
VERSION = "0.1.0"
ARCHIVE_BASENAME = "cyrune-free-v0.1"
SBOM_NAMESPACE_HOST = "cyrune.local"
RELEASE_PREPARATION_METADATA_VERSION = "d7-rc1-rule-fixed.v1"
WEZTERM_SOURCE_PROJECT = "wezterm/wezterm"
WEZTERM_SOURCE_KIND = "github-release-tag"
WEZTERM_SOURCE_EVIDENCE_ORIGIN = "official-github-release"
WEZTERM_SOURCE_PIN = "20240203-110809-5046fc22"
WEZTERM_SOURCE_ARCHIVE = f"wezterm-{WEZTERM_SOURCE_PIN}-src.tar.gz"
WEZTERM_SOURCE_REFERENCE_URL = (
    f"https://github.com/wezterm/wezterm/releases/tag/{WEZTERM_SOURCE_PIN}"
)
PRODUCTIZATION_IDENTITY = {
    "product_line_label": "CYRUNE Terminal",
    "packaged_product_display_name": "CYRUNE",
    "app_bundle_basename": "CYRUNE.app",
    "terminal_bundle_executable_stem": "cyrune",
}

MIT_LICENSE_TEXT = """MIT License

Copyright (c) 2026 CYRUNE contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"""

APACHE_LICENSE_TEXT = """Apache License
Version 2.0, January 2004
http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

   "License" shall mean the terms and conditions for use, reproduction,
   and distribution as defined by Sections 1 through 9 of this document.

   "Licensor" shall mean the copyright owner or entity authorized by
   the copyright owner that is granting the License.

   "Legal Entity" shall mean the union of the acting entity and all
   other entities that control, are controlled by, or are under common
   control with that entity. For the purposes of this definition,
   "control" means (i) the power, direct or indirect, to cause the
   direction or management of such entity, whether by contract or
   otherwise, or (ii) ownership of fifty percent (50%) or more of the
   outstanding shares, or (iii) beneficial ownership of such entity.

   "You" (or "Your") shall mean an individual or Legal Entity
   exercising permissions granted by this License.

   "Source" form shall mean the preferred form for making modifications,
   including but not limited to software source code, documentation
   source, and configuration files.

   "Object" form shall mean any form resulting from mechanical
   transformation or translation of a Source form, including but
   not limited to compiled object code, generated documentation,
   and conversions to other media types.

   "Work" shall mean the work of authorship, whether in Source or
   Object form, made available under the License, as indicated by a
   copyright notice that is included in or attached to the work
   (an example is provided in the Appendix below).

   "Derivative Works" shall mean any work, whether in Source or Object
   form, that is based on (or derived from) the Work and for which the
   editorial revisions, annotations, elaborations, or other modifications
   represent, as a whole, an original work of authorship. For the purposes
   of this License, Derivative Works shall not include works that remain
   separable from, or merely link (or bind by name) to the interfaces of,
   the Work and Derivative Works thereof.

   "Contribution" shall mean any work of authorship, including
   the original version of the Work and any modifications or additions
   to that Work or Derivative Works thereof, that is intentionally
   submitted to Licensor for inclusion in the Work by the copyright owner
   or by an individual or Legal Entity authorized to submit on behalf of
   the copyright owner. For the purposes of this definition, "submitted"
   means any form of electronic, verbal, or written communication sent
   to the Licensor or its representatives, including but not limited to
   communication on electronic mailing lists, source code control systems,
   and issue tracking systems that are managed by, or on behalf of, the
   Licensor for the purpose of discussing and improving the Work, but
   excluding communication that is conspicuously marked or otherwise
   designated in writing by the copyright owner as "Not a Contribution."

   "Contributor" shall mean Licensor and any individual or Legal Entity
   on behalf of whom a Contribution has been received by Licensor and
   subsequently incorporated within the Work.

2. Grant of Copyright License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   copyright license to reproduce, prepare Derivative Works of,
   publicly display, publicly perform, sublicense, and distribute the
   Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   (except as stated in this section) patent license to make, have made,
   use, offer to sell, sell, import, and otherwise transfer the Work,
   where such license applies only to those patent claims licensable
   by such Contributor that are necessarily infringed by their
   Contribution(s) alone or by combination of their Contribution(s)
   with the Work to which such Contribution(s) was submitted. If You
   institute patent litigation against any entity (including a
   cross-claim or counterclaim in a lawsuit) alleging that the Work
   or a Contribution incorporated within the Work constitutes direct
   or contributory patent infringement, then any patent licenses
   granted to You under this License for that Work shall terminate
   as of the date such litigation is filed.

4. Redistribution. You may reproduce and distribute copies of the
   Work or Derivative Works thereof in any medium, with or without
   modifications, and in Source or Object form, provided that You
   meet the following conditions:

   (a) You must give any other recipients of the Work or
       Derivative Works a copy of this License; and

   (b) You must cause any modified files to carry prominent notices
       stating that You changed the files; and

   (c) You must retain, in the Source form of any Derivative Works
       that You distribute, all copyright, patent, trademark, and
       attribution notices from the Source form of the Work,
       excluding those notices that do not pertain to any part of
       the Derivative Works; and

   (d) If the Work includes a "NOTICE" text file as part of its
       distribution, then any Derivative Works that You distribute must
       include a readable copy of the attribution notices contained
       within such NOTICE file, excluding those notices that do not
       pertain to any part of the Derivative Works, in at least one
       of the following places: within a NOTICE text file distributed
       as part of the Derivative Works; within the Source form or
       documentation, if provided along with the Derivative Works; or,
       within a display generated by the Derivative Works, if and
       wherever such third-party notices normally appear. The contents
       of the NOTICE file are for informational purposes only and
       do not modify the License. You may add Your own attribution
       notices within Derivative Works that You distribute, alongside
       or as an addendum to the NOTICE text from the Work, provided
       that such additional attribution notices cannot be construed
       as modifying the License.

   You may add Your own copyright statement to Your modifications and
   may provide additional or different license terms and conditions
   for use, reproduction, or distribution of Your modifications, or
   for any such Derivative Works as a whole, provided Your use,
   reproduction, and distribution of the Work otherwise complies with
   the conditions stated in this License.

5. Submission of Contributions. Unless You explicitly state otherwise,
   any Contribution intentionally submitted for inclusion in the Work
   by You to the Licensor shall be under the terms and conditions of
   this License, without any additional terms or conditions.
   Notwithstanding the above, nothing herein shall supersede or modify
   the terms of any separate license agreement you may have executed
   with Licensor regarding such Contributions.

6. Trademarks. This License does not grant permission to use the trade
   names, trademarks, service marks, or product names of the Licensor,
   except as required for reasonable and customary use in describing the
   origin of the Work and reproducing the content of the NOTICE file.

7. Disclaimer of Warranty. Unless required by applicable law or
   agreed to in writing, Licensor provides the Work (and each
   Contributor provides its Contributions) on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
   implied, including, without limitation, any warranties or conditions
   of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
   PARTICULAR PURPOSE. You are solely responsible for determining the
   appropriateness of using or redistributing the Work and assume any
   risks associated with Your exercise of permissions under this License.

8. Limitation of Liability. In no event and under no legal theory,
   whether in tort (including negligence), contract, or otherwise,
   unless required by applicable law (such as deliberate and grossly
   negligent acts) or agreed to in writing, shall any Contributor be
   liable to You for damages, including any direct, indirect, special,
   incidental, or consequential damages of any character arising as a
   result of this License or out of the use or inability to use the
   Work (including but not limited to damages for loss of goodwill,
   work stoppage, computer failure or malfunction, or any and all
   other commercial damages or losses), even if such Contributor
   has been advised of the possibility of such damages.

9. Accepting Warranty or Additional Liability. While redistributing
   the Work or Derivative Works thereof, You may choose to offer,
   and charge a fee for, acceptance of support, warranty, indemnity,
   or other liability obligations and/or rights consistent with this
   License. However, in accepting such obligations, You may act only
   on Your own behalf and on Your sole responsibility, not on behalf
   of any other Contributor, and only if You agree to indemnify,
   defend, and hold each Contributor harmless for any liability
   incurred by, or claims asserted against, such Contributor by reason
   of your accepting any such warranty or additional liability.

END OF TERMS AND CONDITIONS
"""


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def workspace_root() -> Path:
    return repo_root()


def shipping_root() -> Path:
    return workspace_root() / "target" / "shipping" / "S2"


def run(cmd: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=str(cwd) if cwd else None,
        env=env,
        text=True,
        capture_output=True,
        check=True,
    )


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def sanitize_spdx_id(value: str) -> str:
    cleaned = ["SPDXRef"]
    for char in value:
        if char.isalnum():
            cleaned.append(char)
        else:
            cleaned.append("-")
    return "".join(cleaned)


def cargo_metadata(manifest_path: Path) -> dict[str, Any]:
    result = run(
        [
            "cargo",
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--manifest-path",
            str(manifest_path),
        ],
        cwd=workspace_root(),
    )
    return json.loads(result.stdout)


def write_spdx(metadata: dict[str, Any], out_path: Path, generated_at: str) -> None:
    packages: list[dict[str, Any]] = []
    for package in metadata.get("packages", []):
        name = package["name"]
        version = package["version"]
        package_id = sanitize_spdx_id(f"{name}-{version}")
        license_declared = package.get("license") or "NOASSERTION"
        packages.append(
            {
                "SPDXID": package_id,
                "name": name,
                "versionInfo": version,
                "downloadLocation": "NOASSERTION",
                "licenseConcluded": license_declared,
                "licenseDeclared": license_declared,
                "supplier": "NOASSERTION",
                "filesAnalyzed": False,
            }
        )
    document = {
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": f"{PRODUCT}-{VERSION}",
        "documentNamespace": f"https://cyrune.local/sbom/{ARCHIVE_BASENAME}/{generated_at}",
        "creationInfo": {
            "created": generated_at,
            "creators": ["Tool: stage_shipping_readiness.py"],
        },
        "packages": packages,
    }
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_license_bundle(licenses_dir: Path, sbom_relpath: str) -> None:
    licenses_dir.mkdir(parents=True, exist_ok=True)
    (licenses_dir / "LICENSE-MIT.txt").write_text(MIT_LICENSE_TEXT, encoding="utf-8")
    (licenses_dir / "LICENSE-APACHE-2.0.txt").write_text(APACHE_LICENSE_TEXT, encoding="utf-8")
    notices = textwrap.dedent(
        f"""\
        # THIRD-PARTY NOTICES

        ## First-party workspace

        - CYRUNE Free v0.1 workspace crates: `MIT OR Apache-2.0`

        ## Current productization surface

        - Product line label: `{PRODUCTIZATION_IDENTITY["product_line_label"]}`
        - Packaged product display name: `{PRODUCTIZATION_IDENTITY["packaged_product_display_name"]}`
        - App bundle basename: `{PRODUCTIZATION_IDENTITY["app_bundle_basename"]}`
        - Terminal bundle executable stem: `{PRODUCTIZATION_IDENTITY["terminal_bundle_executable_stem"]}`
        - Current packaged product surface metadata is also carried in `RELEASE_MANIFEST.json.productization_identity`

        ## Dependency inventory

        - Cargo dependency inventory and license expressions are recorded in `{sbom_relpath}`
        - This readiness artifact satisfies third-party notice bundling by shipping this notice file,
          first-party license texts, and the SPDX dependency inventory together

        ## WezTerm integration boundary

        - This readiness artifact ships the canonical `wezterm.lua` template only
        - No WezTerm binary is bundled in this readiness artifact
        - If a later user-facing package bundles WezTerm, MIT notice and attribution must be added here
        """
    )
    (licenses_dir / "THIRD-PARTY-NOTICES.md").write_text(notices, encoding="utf-8")


def copy_tree(src: Path, dst: Path) -> None:
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(src, dst)


def overlay_tree(src: Path, dst: Path) -> None:
    if not src.exists():
        raise FileNotFoundError(f"overlay source is missing: {src}")
    for path in sorted(src.rglob("*")):
        relative = path.relative_to(src)
        destination = dst / relative
        if path.is_dir():
            destination.mkdir(parents=True, exist_ok=True)
            continue
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, destination)


def make_executable(path: Path) -> None:
    mode = path.stat().st_mode
    path.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def maybe_codesign(binary_path: Path, guard_dir: Path) -> str:
    if platform.system() != "Darwin":
        return "hash-only"
    run(["codesign", "--force", "--sign", "-", str(binary_path)])
    verification = run(["codesign", "--verify", "--verbose=2", str(binary_path)])
    guard_dir.mkdir(parents=True, exist_ok=True)
    (guard_dir / f"{binary_path.name}-codesign.txt").write_text(
        verification.stdout + verification.stderr,
        encoding="utf-8",
    )
    return "macos-adhoc"


def copy_adapter_resolution_assets(*, adapter_root: Path, staging_home: Path, bundle_root: Path) -> None:
    adapter_bundle_root = bundle_root / "adapter"
    copy_tree(adapter_root / "catalog", adapter_bundle_root / "catalog")
    copy_tree(adapter_root / "policies", adapter_bundle_root / "policies")
    copy_tree(adapter_root / "bindings", adapter_bundle_root / "bindings")

    registry_src = staging_home / "registry" / "execution-adapters" / "approved"
    runtime_ipc_src = staging_home / "runtime" / "ipc"
    if not registry_src.exists():
        raise FileNotFoundError(f"approved registry assets are missing: {registry_src}")
    if not runtime_ipc_src.exists():
        raise FileNotFoundError(f"runtime ipc assets are missing: {runtime_ipc_src}")

    registry_dst = bundle_root / "registry" / "execution-adapters" / "approved"
    profiles_src = registry_src / "profiles"
    profiles_dst = registry_dst / "profiles"
    profiles_dst.mkdir(parents=True, exist_ok=True)
    shutil.copy2(registry_src / "registry.json", registry_dst / "registry.json")
    copy_tree(runtime_ipc_src, bundle_root / "runtime" / "ipc")

    for profile_src in sorted(profiles_src.glob("*.json")):
        profile = json.loads(profile_src.read_text(encoding="utf-8"))
        raw_launcher_path = profile.get("launcher_path")
        if not isinstance(raw_launcher_path, str) or not raw_launcher_path:
            raise ValueError(f"launcher_path is missing in {profile_src}")

        launcher_source = Path(raw_launcher_path)
        if launcher_source.is_absolute():
            try:
                launcher_relative = launcher_source.relative_to(staging_home)
            except ValueError as exc:
                raise ValueError(
                    f"launcher_path must resolve under staging_home for bundle payload: {raw_launcher_path}"
                ) from exc
        else:
            launcher_relative = launcher_source
            launcher_source = staging_home / launcher_relative

        if not launcher_source.exists():
            raise FileNotFoundError(f"launcher source is missing: {launcher_source}")

        bundled_launcher = bundle_root / launcher_relative
        if not bundled_launcher.exists():
            raise FileNotFoundError(f"bundled launcher is missing: {bundled_launcher}")

        profile["launcher_path"] = launcher_relative.as_posix()
        profile_dst = profiles_dst / profile_src.name
        profile_dst.write_text(json.dumps(profile, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_bundle_root(
    package_root: Path,
    *,
    adapter_root: Path,
    staging_home: Path,
    bundle_resources_root: Path,
) -> None:
    bundle_root = package_root / "share" / "cyrune" / "bundle-root"
    copy_adapter_resolution_assets(
        adapter_root=adapter_root,
        staging_home=staging_home,
        bundle_root=bundle_root,
    )
    terminal_templates_dir = bundle_root / "terminal" / "templates"
    terminal_templates_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(
        staging_home / "terminal" / "config" / "wezterm.lua",
        terminal_templates_dir / "wezterm.lua",
    )
    overlay_tree(bundle_resources_root, bundle_root)


def write_release_manifest(
    out_path: Path,
    *,
    generated_at: str,
    signature_mode: str,
) -> None:
    manifest = {
        "product": PRODUCT,
        "version": VERSION,
        "generated_at": generated_at,
        "distribution_unit": f"{ARCHIVE_BASENAME}.tar.gz",
        "package_root": ARCHIVE_BASENAME,
        "primary_os": "macOS",
        "runtime_entry": "bin/cyr",
        "daemon_entry": "bin/cyrune-daemon",
        "bundle_root_path": "share/cyrune/bundle-root",
        "home_template_path": "share/cyrune/home-template",
        "terminal_config_path": "share/cyrune/home-template/terminal/config/wezterm.lua",
        "productization_identity": dict(PRODUCTIZATION_IDENTITY),
        "license_bundle_path": "share/licenses",
        "sbom_path": "share/sbom/cyrune-free-v0.1.spdx.json",
        "integrity_mode": "sha256",
        "signature_mode": signature_mode,
        "update_policy": "fixed-distribution/no-self-update",
        "upstream_intake_mode": "evidence-based",
        "upstream_follow_triggers": ["security", "critical_bug", "required_feature"],
        "upstream_auto_follow": False,
    }
    out_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def normalize_identifier_suffix(label: str) -> str:
    tokens = [
        "".join(char for char in chunk.lower() if char.isalnum())
        for chunk in label.split()
    ]
    tokens = [token for token in tokens if token]
    if not tokens:
        raise ValueError("product line label cannot derive reverse-DNS suffix")
    return tokens[-1]


def derive_reverse_dns_bundle_identifier(product_line_label: str) -> str:
    namespace_root = ".".join(reversed(SBOM_NAMESPACE_HOST.split(".")))
    return f"{namespace_root}.{normalize_identifier_suffix(product_line_label)}"


def write_release_preparation_metadata(
    out_path: Path,
    *,
    primary_os: str,
    distribution_unit: str,
) -> None:
    payload = {
        "metadata_version": RELEASE_PREPARATION_METADATA_VERSION,
        "reverse_dns_bundle_identifier": derive_reverse_dns_bundle_identifier(
            PRODUCTIZATION_IDENTITY["product_line_label"]
        ),
        "installer_artifact": {
            "artifact_class": "app_bundle",
            "platform": primary_os,
            "emitted_name": PRODUCTIZATION_IDENTITY["app_bundle_basename"],
        },
        "archive_artifact": {
            "artifact_class": "distribution_archive",
            "platform": primary_os,
            "emitted_name": distribution_unit,
        },
        "upstream_source_pin": {
            "source_project": WEZTERM_SOURCE_PROJECT,
            "source_kind": WEZTERM_SOURCE_KIND,
            "exact_revision": WEZTERM_SOURCE_PIN,
            "source_archive": WEZTERM_SOURCE_ARCHIVE,
            "evidence_origin": WEZTERM_SOURCE_EVIDENCE_ORIGIN,
            "source_reference_url": WEZTERM_SOURCE_REFERENCE_URL,
            "upstream_intake_mode": "evidence-based",
            "upstream_follow_triggers": ["security", "critical_bug", "required_feature"],
            "upstream_auto_follow": False,
        },
    }
    out_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_hash_list(package_root: Path) -> None:
    lines: list[str] = []
    for path in sorted(package_root.rglob("*")):
        if path.is_file():
            rel = path.relative_to(package_root)
            lines.append(f"{sha256_file(path)}  {rel.as_posix()}")
    (package_root / "SHA256SUMS.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


def create_archive(package_root: Path, archive_path: Path) -> None:
    with tarfile.open(archive_path, "w:gz") as archive:
        archive.add(package_root, arcname=package_root.name)


def main() -> int:
    root = repo_root()
    workspace = workspace_root()
    output_root = shipping_root()
    guard_dir = output_root / "guard"
    package_root = output_root / ARCHIVE_BASENAME
    archive_path = output_root / f"{ARCHIVE_BASENAME}.tar.gz"
    generated_at = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")

    if output_root.exists():
        shutil.rmtree(output_root)
    guard_dir.mkdir(parents=True, exist_ok=True)

    manifest_path = workspace / "Cargo.toml"
    run(
        [
            "cargo",
            "build",
            "--release",
            "--manifest-path",
            str(manifest_path),
            "--bin",
            "cyrune-daemon",
            "--bin",
            "cyrune-runtime-cli",
        ],
        cwd=workspace,
    )

    release_dir = workspace / "target" / "release"
    runtime_bin = release_dir / "cyrune-runtime-cli"
    daemon_bin = release_dir / "cyrune-daemon"
    if not runtime_bin.exists() or not daemon_bin.exists():
        raise FileNotFoundError("release binaries are missing after cargo build --release")

    staging_home = output_root / "staging-home"
    adapter_root = workspace / "Adapter" / "v0.1" / "0"
    bundle_resources_root = workspace / "resources" / "bundle-root"
    env = os.environ.copy()
    env["CYRUNE_HOME"] = str(staging_home)
    env["CRANE_ROOT"] = str(workspace)
    env["CYRUNE_DAEMON_BIN"] = str(daemon_bin)
    doctor = run([str(runtime_bin), "doctor"], cwd=workspace, env=env)
    doctor_health = json.loads(doctor.stdout)
    (guard_dir / "doctor-health.json").write_text(
        json.dumps(doctor_health, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    (package_root / "bin").mkdir(parents=True, exist_ok=True)
    (package_root / "share" / "cyrune").mkdir(parents=True, exist_ok=True)
    shutil.copy2(runtime_bin, package_root / "bin" / "cyr")
    shutil.copy2(daemon_bin, package_root / "bin" / "cyrune-daemon")
    make_executable(package_root / "bin" / "cyr")
    make_executable(package_root / "bin" / "cyrune-daemon")

    copy_tree(staging_home, package_root / "share" / "cyrune" / "home-template")
    overlay_tree(bundle_resources_root, package_root / "share" / "cyrune" / "home-template")
    write_bundle_root(
        package_root,
        adapter_root=adapter_root,
        staging_home=staging_home,
        bundle_resources_root=bundle_resources_root,
    )

    metadata = cargo_metadata(manifest_path)
    sbom_path = package_root / "share" / "sbom" / "cyrune-free-v0.1.spdx.json"
    write_spdx(metadata, sbom_path, generated_at)
    write_license_bundle(package_root / "share" / "licenses", "share/sbom/cyrune-free-v0.1.spdx.json")

    signature_modes = {
        "cyr": maybe_codesign(package_root / "bin" / "cyr", guard_dir),
        "cyrune-daemon": maybe_codesign(package_root / "bin" / "cyrune-daemon", guard_dir),
    }
    if len(set(signature_modes.values())) == 1:
        signature_mode = next(iter(signature_modes.values()))
    else:
        signature_mode = json.dumps(signature_modes, sort_keys=True)

    write_release_manifest(
        package_root / "RELEASE_MANIFEST.json",
        generated_at=generated_at,
        signature_mode=signature_mode,
    )
    write_release_preparation_metadata(
        package_root / "RELEASE_PREPARATION.json",
        primary_os="macOS",
        distribution_unit=f"{ARCHIVE_BASENAME}.tar.gz",
    )

    write_hash_list(package_root)
    create_archive(package_root, archive_path)
    (guard_dir / "archive-sha256.txt").write_text(
        f"{sha256_file(archive_path)}  {archive_path.name}\n",
        encoding="utf-8",
    )

    result = {
        "product": PRODUCT,
        "version": VERSION,
        "package_root": str(package_root),
        "archive_path": str(archive_path),
        "doctor_health_path": str(guard_dir / "doctor-health.json"),
        "signature_mode": signature_mode,
    }
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
