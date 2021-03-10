#!/bin/bash

case $1 in
    help)
        printf 'No\n'
        ;;

    update-version)
        sed 's/version = "0.0.0"/version = "'$2'"/g' Cargo.toml > Cargo.toml.tmp
        mv Cargo.toml.tmp Cargo.toml
        sed 's/pkgver=0.0.0/pkgver='$2'/g' pkg/aur/PKGBUILD > pkg/aur/PKGBUILD.tmp
        mv pkg/aur/PKGBUILD.tmp pkg/aur/PKGBUILD
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
