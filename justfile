build:
    cargo build

build-release:
    cargo build --release

update-tests:
    #!/usr/bin/env sh
    version='0.2'
    if ! [ "$(cctestgen --version | grep -F $version)" ]; then
        cargo install --force --git https://github.com/nathanwhit/cctestgen
    fi
    for file in ./descriptors/*;
    do 
        base=$(basename $file)
        testname="$(echo $base | sed 's/\([A-Z]\)/_\L\1/g;s/^_//').rs"
        testfile=./tests/$testname
        unittestfile=./src/handler/tests/execution/$testname
        /home/nathanw/cctestgen/target/release/cctestgen --mode=integration $file > $testfile
        /home/nathanw/cctestgen/target/release/cctestgen --mode=unit $file > $unittestfile
    done
    cargo fmt

integration_options := "'old-sawtooth integration-testing'"

integration-test: update-tests
    cargo test --features {{integration_options}} --test '*' --no-fail-fast

unit-test: update-tests
    cargo test

test: update-tests
    cargo test --features {{integration_options}} --no-fail-fast