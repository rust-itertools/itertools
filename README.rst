
Itertools
=========

Extra iterator adaptors, functions and macros.

Please read the `API documentation here`__

__ http://bluss.github.io/rust-itertools/

|build_status|_ |crates|_

.. |build_status| image:: https://travis-ci.org/bluss/rust-itertools.svg?branch=master
.. _build_status: https://travis-ci.org/bluss/rust-itertools

.. |crates| image:: http://meritbadge.herokuapp.com/itertools
.. _crates: https://crates.io/crates/itertools

How to use with cargo::

    [dependencies]
    itertools = "0.3"

How to use in your crate::

    #[macro_use]
    extern crate itertools;

    use itertools::Itertools;


Recent Changes
--------------

- 0.3.13

  - Added adaptor .interleave_shortest()
  - Added adaptor .pad_using()

- 0.3.11

  - Added assert_equal function

- 0.3.10

  - Bugfix .combinations() size_hint.

- 0.3.8

  - Added source RepeatCall

- 0.3.7

  - Added adaptor PutBackN
  - Added adaptor .combinations()

- 0.3.6

  - Added itertools::partition, partition a sequence in place based on a predicate.
  - Deprecate icompr!() with no replacement.

- 0.3.5

  - .map_fn() replaces deprecated .fn_map().

- 0.3.4

  - .take_while_ref() *by-ref adaptor*
  - .coalesce() *adaptor*
  - .mend_slices() *adaptor*

- 0.3.3

  - .dropping_back() *method*
  - .fold1() *method*
  - .is_empty_hint() *method*

License
-------

Dual-licensed to be compatible with the Rust project.

Licensed under the Apache License, Version 2.0
http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
http://opensource.org/licenses/MIT, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
