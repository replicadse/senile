#!/bin/bash

case $1 in
    help)
        printf 'No\n'
        ;;

    update-version)
        sed 's/version = "0.0.0"/version = "'$2'"/g' Cargo.toml > Cargo.toml.tmp
        mv Cargo.toml.tmp Cargo.toml
        ;;

    install-hooks)
        # install hooks
        rm -rf .git/hooks
        ln -s ../scripts/git-hooks .git/hooks
        chmod -R +x ./scripts/*
        ;;

    scan)
        #cargo clippy --all-targets --all-features -- -D warnings
        cargo fmt --all -- --check
        #cargo sync-readme -c
        ;;
esac
