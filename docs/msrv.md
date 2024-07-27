# Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on all stable Rust versions going back to
the version stated as MSRV in the main
[README](https://github.com/EngJay/pa-spl). It *might* compile with even older
versions but that may change in any new patch release.

## How the MSRV will be upgraded

For this library, we do not consider upgrading the MSRV a strictly breaking
change as defined by [SemVer](https://semver.org).

The following rules will be followed when upgrading it:

MSRV

- will not be updated on any patch release: \_.\_.*Z*.
- may be upgraded on any *major* or *minor* release: *X*.*Y*.\_.
- may be upgraded in any preliminary version release (e.g. an `-alpha` release)
  as these serve as preparation for the final release.
- upgrades will be clearly stated in the changelog.

This applies both to `0._._` releases as well as `>=1._._` releases.

For example:

For a given `x.y.z` release, we may upgrade the MSRV on `x` and `y` releases but
not on `z` releases.

If your MSRV upgrade policy differs from this, you are advised to specify the
dependency in your `Cargo.toml` accordingly.

See the
[Rust Embedded Working Group MSRV RFC](https://github.com/rust-embedded/wg/blob/master/rfcs/0523-msrv-2020.md)
for more background information and reasoning.
