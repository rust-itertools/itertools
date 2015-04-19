
docs:
	cargo doc --no-deps
	rm -rf ./doc
	cp -r ./target/doc ./doc

.PHONY: docs
