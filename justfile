build:
    cargo build

build-release:
    cargo build --release

update-tests:
    #!/usr/bin/env sh
    version='0.3'
    if ! [ "$(cctestgen --version | grep -F $version)" ]; then
        cargo install --force --git https://github.com/nathanwhit/cctestgen
    fi
    integration_main=./tests/integration/main.rs
    echo 'pub mod common;' > $integration_main
    for file in ./descriptors/*;
    do 
        base=$(basename $file)
        # convert from PascalCase to snake_case
        filename="$(echo $base | sed 's/\([A-Z]\)/_\L\1/g;s/^_//')"
        testname="${filename%.*}"
        testfile=./tests/integration/$testname.rs
        unittestfile=./src/handler/tests/execution/$testname.rs
        cctestgen --mode=integration $file > $testfile
        cctestgen --mode=unit $file > $unittestfile
        echo "mod ${testname};\n" >> $integration_main
    done
    cargo fmt
    # cargo clippy --fix --allow-dirty

integration_options := "'old-sawtooth integration-testing'"

integration-test: update-tests
    cargo test --features {{integration_options}} --test '*' --no-fail-fast

unit-test: update-tests
    cargo test

test: update-tests
    cargo test --features {{integration_options}} --no-fail-fast

run-test:
    cargo test --features {{integration_options}} --no-fail-fast

run-unit-test:
    cargo test

test-matching TEST: update-tests
    cargo test --features {{integration_options}} {{TEST}}

test-matching-integration TEST: update-tests
    cargo test --features {{integration_options}} --test '{{TEST}}'

coverage: update-tests
    cargo llvm-cov --features {{integration_options}} --no-fail-fast --html