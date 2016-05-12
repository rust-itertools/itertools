
Itertools
=========

Extra iterator adaptors, functions and macros. Requires Rust 1.2+.

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

- 0.4.15

  - Fixup on top of the workaround in 0.4.14. A function in itertools::free was
    removed by mistake and now it is added back again.

- 0.4.14

  - Workaround an upstream regression in a rust nightly build that broke
    compilation of of itertools::free::{interleave, merge}

- 0.4.13

  - Add .minmax() and .minmax_by_key(), iterator methods for finding both minimum
    and maximum in one scan.
  - Add .format_default(), a simpler version of .format() (lazy formatting
    for iterators).

- 0.4.12

  - Add .zip_eq(), an adaptor like .zip() except it ensures iterators
    of inequal length don't pass silently (instead it panics).
  - Add .fold_while(), an iterator method that is a fold that
    can short-circuit.
  - Add .partition_map(), an iterator method that can separate elements
    into two collections.

- 0.4.11

  - Add .get() for Stride{,Mut} and .get_mut() for StrideMut

- 0.4.10

  - Improve performance of .kmerge()

- 0.4.9

  - Add k-ary merge adaptor .kmerge()
  - Fix a bug in .islice() with ranges a..b where a > b.

- 0.4.8

  - Implement Clone, Debug for Linspace

- 0.4.7

  - Add function diff_with() that compares two iterators
  - Add .combinations_n(), an n-ary combinations iterator
  - Add methods PutBack::with_value and PutBack::into_parts.

- 0.4.6

  - Add method .sorted()
  - Add module ``itertools::free`` with free function variants of common
    iterator adaptors and methods.
    For example ``enumerate(iterable)``, ``rev(iterable)``, and so on.

- 0.4.5

  - Add .flatten()

- 0.4.4

  - Allow composing ZipSlices with itself

- 0.4.3

  - Write iproduct!() as a single expression; this allows temporary values
    in its arguments.

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
