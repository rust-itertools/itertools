#!/bin/sh

# build with all feature flags in the nightly

FEATURES=

if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
    FEATURES="--features unstable"
fi

exec cargo "$@" $FEATURES
