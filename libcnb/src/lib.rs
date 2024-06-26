#![doc = include_str!("../README.md")]

pub mod build;
pub mod detect;
pub mod exec_d;
pub mod generic;
pub mod layer;
pub mod layer_env;
pub mod sbom;

// Internals that need to be public for macros
#[doc(hidden)]
pub mod internals;

mod buildpack;
mod env;
mod error;
mod exit_code;
mod platform;
mod runtime;
mod target;
#[cfg(feature = "trace")]
mod tracing;
mod util;

pub use buildpack::Buildpack;
pub use env::*;
pub use error::*;
pub use libcnb_common::toml_file::*;
pub use platform::*;
pub use runtime::*;
pub use target::*;

#[cfg(all(test, not(feature = "trace")))]
use serde_json as _;

/// Provides types for CNB data formats. Is a re-export of the `libcnb-data` crate.
#[doc(inline)]
pub use libcnb_data as data;

const LIBCNB_SUPPORTED_BUILDPACK_API: data::buildpack::BuildpackApi =
    data::buildpack::BuildpackApi {
        major: 0,
        minor: 10,
    };

/// Generates a main function for the given buildpack.
///
/// It will create the main function and wires up the buildpack to the framework.
///
/// # Example:
/// ```
/// use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
/// use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
/// use libcnb::generic::{GenericError, GenericMetadata, GenericPlatform};
/// use libcnb::{buildpack_main, Buildpack};
///
/// pub(crate) struct MyBuildpack;
///
/// impl Buildpack for MyBuildpack {
///     type Platform = GenericPlatform;
///     type Metadata = GenericMetadata;
///     type Error = GenericError;
///
///     fn detect(
///         &self,
///         context: DetectContext<Self>,
///     ) -> libcnb::Result<DetectResult, Self::Error> {
///         DetectResultBuilder::pass().build()
///     }
///
///     fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
///         BuildResultBuilder::new().build()
///     }
/// }
///
/// buildpack_main!(MyBuildpack);
/// ```
#[macro_export]
macro_rules! buildpack_main {
    ($buildpack:expr) => {
        fn main() {
            ::libcnb::libcnb_runtime(&$buildpack);
        }
    };
}

/// Resolves the path to an additional buildpack binary by Cargo target name.
///
/// This can be used to copy additional binaries to layers or use them for exec.d.
///
/// To add an additional binary to a buildpack, add a new file with a main function to `bin/`.
/// Cargo will [automatically configure it as a binary target](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#target-auto-discovery)
/// with the name of file.
///
/// **Note**: This only works properly if the buildpack is packaged with `libcnb-cargo`/`libcnb-test`.
///
/// ```no_run,compile_fail
/// use libcnb::additional_buildpack_binary_path;
/// # let layer_dir = std::path::PathBuf::from(".");
///
/// std::fs::copy(
///     // This would not compile in this doctest since there is no `runtime_tool` binary target.
///     additional_buildpack_binary_path!("runtime_tool"),
///     layer_dir.join("runtime_tool"),
/// )
/// .unwrap();
/// ```
#[macro_export]
macro_rules! additional_buildpack_binary_path {
    ($target_name:expr) => {
        ::libcnb::internals::verify_bin_target_exists!(
            $target_name,
            {
                ::std::env::var("CNB_BUILDPACK_DIR")
                    .map(::std::path::PathBuf::from)
                    .expect("Couldn't read CNB_BUILDPACK_DIR environment variable")
                    .join(".libcnb-cargo")
                    .join("additional-bin")
                    .join($target_name)
            },
            {
                compile_error!(concat!(
                    $target_name,
                    " is not a valid binary target in this buildpack crate!"
                ))
            }
        )
    };
}
