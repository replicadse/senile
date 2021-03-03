#!/bin/bash

case $1 in
    help)
        printf 'No\n'
        ;;

    update-version)
        sed 's/version = "0.0.0"/version = "'$2'"/g' Cargo.toml > Cargo.toml.tmp
        mv Cargo.toml.tmp Cargo.toml
        ;;

    init)
        # install hooks
        rm -rf .git/hooks
        ln -s ../scripts/git-hooks .git/hooks
        chmod -R +x ./scripts/*
        # install tools
        cargo install cargo-sync-readme
        ;;

    cover)
        export coverflags CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off"
        $coverflags cargo +nightly build
        $coverflags cargo +nightly test
        grcov ./target/debug/ -s . -t lcov --llvm --ignore-not-existing -o ./target/debug/coverage
        genhtml -o ./target/debug/coverage-html --show-details --highlight ./target/debug/coverage
        ;;

    scan)
        cargo clippy --all-targets --all-features -- -D warnings
        cargo fmt --all -- --check
        cargo sync-readme -c
        ;;
esac
