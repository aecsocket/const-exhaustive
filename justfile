default:
    just --list

check:
    #!/usr/bin/env bash
    set +e
    failed=0

    typos . || failed=1
    tombi fmt --check . || failed=1
    tombi lint . || failed=1
    cargo +nightly fmt --check || failed=1
    cargo shear --deny-warnings || failed=1
    cargo clippy --workspace --all-features --all-targets || failed=1

    if [ "$failed" -ne 0 ]; then exit 1; fi

prepare:
    #!/usr/bin/env bash
    set +e

    tombi fmt .
    cargo +nightly fmt
    cargo shear --fix
    just check

test:
    cargo +nightly miri nextest run --workspace --all-features --all-targets
