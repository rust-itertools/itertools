
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
    itertools = "0.4"

How to use in your crate:

.. code:: rust

    #[macro_use] extern crate itertools;

    use itertools::Itertools;


Recent Changes
--------------

- 0.4.2

  - Add .fold_options()
  - Require Rust 1.1 or later

- 0.4.1

  - Update .dropping() to take advantage of .nth()

- 0.4.0

  - .merge(), .unique() and .dedup() now perform better due to not using
    function pointers
  - Add free functions enumerate() and rev()
  - Breaking changes:

    - Return types of .merge() and .merge_by() renamed and changed
    - Method Merge::new removed
    - .merge_by() now takes a closure that returns bool.
    - Return type of .dedup() changed
    - Return type of .mend_slices() changed
    - Return type of .unique() changed
    - Removed function times(), struct Times: use a range instead
    - Removed deprecated macro icompr!()
    - Removed deprecated FnMap and method .fn_map(): use .map_fn()
    - .interleave_shortest() is no longer guaranteed to act like fused

- 0.3.25

  - Rename .sort_by() to .sorted_by(). Old name is deprecated.
  - Fix well-formedness warnings from RFC 1214, no user visible impact

- 0.3.24

  - Improve performance of .merge()'s ordering function slightly

- 0.3.23

  - Added .chunks_lazy(), similar to (and based on) .group_by_lazy().
  - Tweak linspace to match numpy.linspace and make it double ended.

- 0.3.22

  - Added ZipSlices, a fast zip for slices

- 0.3.21

  - Remove `Debug` impl for `Format`, it will have different use later

- 0.3.20

  - Optimize .group_by_lazy()

- 0.3.19

  - Added .group_by_lazy(), a possibly nonallocating group by
  - Added .format(), a nonallocating formatting helper for iterators
  - Remove uses of RandomAccessIterator since it has been deprecated in rust.

- 0.3.17

  - Added (adopted) Unfold from rust

- 0.3.16

  - Added adaptors .unique(), .unique_by()

- 0.3.15

  - Added method .sort_by()

- 0.3.14

  - Added adaptor .while_some()

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
