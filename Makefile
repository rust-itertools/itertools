
docs: VERSION
	cargo doc --no-deps
	rm -rf ./doc
	cp -r ./target/doc ./doc
	# Put in the crate version into the docs
	find ./doc -name "*.html" -exec sed -i -e "s/<title>\(.*\) - Rust/<title>itertools $(shell cat VERSION) - \1 - Rust/g" {} \;

VERSION: Cargo.toml
	cargo pkgid | sed -e "s/.*#//" > VERSION

.PHONY: docs
