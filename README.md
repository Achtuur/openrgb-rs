openrgb-rs2 [![crates.io](https://img.shields.io/crates/v/openrgb.svg)](https://crates.io/crates/openrgb) [![tests](https://github.com/nicoulaj/openrgb-rs/actions/workflows/tests.yml/badge.svg)](https://github.com/nicoulaj/openrgb-rs/actions/workflows/tests.yml)
==========

**Rust client library for [OpenRGB SDK](https://openrgb.org).**

See [documentation](https://docs.rs/openrgb) and [examples](https://github.com/nicoulaj/openrgb-rs/tree/master/examples).

# Original openrgb-rs

This repository was previously maintained by nicoulaj. I have attempted to reach out to them, but received no response. As a result I decided to republish the OpenRGB SDK under a new name (`openrgb-rs2`). This makes updating the sdk hard.

## Whats different?

Support for OpenRGB protocol versions 4 and 5 is added. There's also now a friendlier to use API than before.

Internally there's some changes in how serializing/deserializing the protocol is done. I decided it was easier to read/write to a buffer, rather than directly to a stream as was previously done. For the end user there should not be much visible change though. I have not done any benchmarking, so I'm not sure about the performance. I can update my entire rig at about 300 FPS at release mode, so I'm not too worried about performance anyway.