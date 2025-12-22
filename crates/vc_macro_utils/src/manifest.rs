use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{PoisonError, RwLock};
use std::time::SystemTime;

use toml_edit::{Document, Item, Table};

/// Locate an accessible [`syn::Path`] for another crate as seen from the
/// caller's Cargo.toml.
///
/// This helper is intended for proc-macro code
/// generation where the emitted path must be valid from the invoking crate.
///
/// # Example
///
/// ```rust
/// # use vc_macro_utils::Manifest;
/// let p: syn::Path = Manifest::shared(|m| m.get_crate_path("vc_reflect"));
/// ```
///
/// The cost of this operation is not low, and the caller should storage the results
/// and try to call only once per proc-macro.
///
/// # Resolution rules
///
/// 1. If the requested crate is listed in `dependencies`, return `::crate_name`.
/// 2. If requested crate name begins with `vc_`, and target crate depends on
///    the workspace crate `voidcraft`,  return `::voidcraft::short_name`
///    (e.g. `vc_cfg` -> `::voidcraft::cfg`).
/// 3. If requested crate name begins with `vc_`, and target crate depends on
///    the workspace crate `vc_core` ,  return `::vc_core::short_name`
/// 4. If requested crate name begins with `vc_`, and target crate depends on
///    alias `vc`, return `::vc::short_name`.
/// 5. Repeat step 1-4 in `dev-dependencies`.
/// 6. Otherwise, fall back to the absolute path `::crate_name`.
///
/// ## Note
/// When a crate needs to reference itself, library code should use
/// `crate::...`, while doctests and other external code typically use the
/// absolute path `::crate_name`.
///
/// To support both cases adding an alias such as
/// `extern crate self as vc_reflect;` in the crate root can resolve the conflict.
#[derive(Debug)]
pub struct Manifest {
    pub manifest: Document<Box<str>>,
    pub modified_time: SystemTime,
}

const FULL_ENGINE_NAME: &str = "voidcraft";
const CORE_ENGINE_NAME: &str = "vc_core";
const SHORT_ENGINE_NAME: &str = "vc";
const ENGINE_PREFIX: &str = "vc_";

impl Manifest {
    // Try get `Cargo.toml` path.
    #[inline(never)]
    fn get_manifest_path() -> PathBuf {
        env::var_os("CARGO_MANIFEST_DIR")
            .map(|path| {
                let mut path = PathBuf::from(path);
                path.push("Cargo.toml");
                assert!(
                    path.exists(),
                    "Cargo manifest does not exist at path {}",
                    path.display(),
                );
                path
            })
            .expect("CARGO_MANIFEST_DIR should be auto-defined by cargo.")
    }

    // Try get `Cargo.toml` modified time.
    #[inline(never)]
    fn get_manifest_modified_time(
        cargo_manifest_path: &Path,
    ) -> Result<SystemTime, std::io::Error> {
        std::fs::metadata(cargo_manifest_path).and_then(|metadata| metadata.modified())
    }

    #[inline(never)]
    fn read_manifest(path: &Path) -> Document<Box<str>> {
        let manifest = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Unable to read cargo manifest: {}", path.display()))
            .into_boxed_str();
        Document::parse(manifest)
            .unwrap_or_else(|_| panic!("Failed to parse cargo manifest: {}", path.display()))
    }

    // Attempt to parse the provided path as a syntax tree node.
    #[inline]
    fn parse_str<T: syn::parse::Parse>(path: &str) -> T {
        syn::parse_str(path).unwrap()
    }

    #[inline]
    fn find_in_deps(deps: &Table, name: &str) -> Option<syn::Path> {
        if deps.contains_key(name) {
            // This dependency exists in this crate
            Some(Self::parse_str(&format!("::{name}")))
        } else {
            if let Some(module) = name.strip_prefix(ENGINE_PREFIX) {
                if deps.contains_key(FULL_ENGINE_NAME) {
                    let mut path = Self::parse_str::<syn::Path>(&format!("::{FULL_ENGINE_NAME}"));
                    path.segments.push(Self::parse_str(module));
                    return Some(path);
                }
                if deps.contains_key(CORE_ENGINE_NAME) {
                    let mut path = Self::parse_str::<syn::Path>(&format!("::{CORE_ENGINE_NAME}"));
                    path.segments.push(Self::parse_str(module));
                    return Some(path);
                }
                if deps.contains_key(SHORT_ENGINE_NAME) {
                    let mut path = Self::parse_str::<syn::Path>(&format!("::{SHORT_ENGINE_NAME}"));
                    path.segments.push(Self::parse_str(module));
                    return Some(path);
                }
            }
            None
        }
    }

    /// Return a [`syn::Path`] for the package named `name` as resolved from this
    /// crate's Cargo.toml. See the top-level documentation for the resolution
    /// order and examples.
    #[inline(never)]
    pub fn get_crate_path(&self, name: &str) -> syn::Path {
        if let Some(Item::Table(deps)) = self.manifest.get("dependencies")
            && let Some(val) = Self::find_in_deps(deps, name)
        {
            return val;
        }

        if let Some(Item::Table(deps)) = self.manifest.get("dev-dependencies")
            && let Some(val) = Self::find_in_deps(deps, name)
        {
            return val;
        }

        Self::parse_str(&format!("::{name}"))
    }

    /// Obtain the [Manifest] of the caller's Cargo.toml.
    ///
    /// This function reads and caches the caller's `Cargo.toml`. Parsing the
    /// manifest and acquiring the global cache lock are relatively expensive for
    /// proc-macros, so callers should invoke [`Manifest::shared`] sparingly (typically
    /// once per macro invocation) and cache the returned [`syn::Path`] where possible.
    pub fn shared<R>(func: impl FnOnce(&Self) -> R) -> R {
        static MANIFESTS: RwLock<BTreeMap<PathBuf, Manifest>> = RwLock::new(BTreeMap::new());

        let manifest_path = Self::get_manifest_path();
        let modified_time = Self::get_manifest_modified_time(&manifest_path)
            .expect("The Cargo.toml should have a modified time.");

        let manifests = MANIFESTS.read().unwrap_or_else(PoisonError::into_inner);

        if let Some(manifest) = manifests.get(&manifest_path)
            && manifest.modified_time == modified_time
        {
            return func(manifest);
        }

        drop(manifests);

        let manifest = Manifest {
            manifest: Self::read_manifest(&manifest_path),
            modified_time,
        };

        let result = func(&manifest);

        MANIFESTS
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(manifest_path, manifest);

        result
    }
}
