# TROUBLESHOOTING

These remediation notes are limited to the three public scripts and do not extend to internal or native-distribution workflows.

## prepare-public-run.sh

If this step fails, confirm the exact release asset URL is reachable, confirm carrier download and extraction succeeded, then rerun ./scripts/prepare-public-run.sh.

## doctor.sh

If this step fails, rerun ./scripts/prepare-public-run.sh first, then rerun ./scripts/doctor.sh.

## first-success.sh

If this step fails, rerun ./scripts/prepare-public-run.sh, confirm ./scripts/doctor.sh passes, then rerun ./scripts/first-success.sh.
