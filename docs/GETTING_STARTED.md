# GETTING_STARTED

Run the three scripts in order from the tracked public branch surface. `prepare-public-run.sh` downloads the exact release asset `cyrune-free-v0.1.tar.gz`, normalizes the required non-tracked carrier into `target/public-run/`, and then prepares the local runtime state. Do not skip steps or change the sequence.

## 1. prepare-public-run.sh

```bash
./scripts/prepare-public-run.sh
```

## 2. doctor.sh

```bash
./scripts/doctor.sh
```

## 3. first-success.sh

```bash
./scripts/first-success.sh
```
