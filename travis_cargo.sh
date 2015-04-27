#!/bin/sh

# build with all feature flags in the nightly

FEATURES=

if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
    FEATURES="unstable qc"
else
    if [ "$1" = "bench" ]; then exit 0; fi
fi

exec cargo "$@" --features "$FEATURES"
