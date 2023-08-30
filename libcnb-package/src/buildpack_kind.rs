use libcnb_common::toml_file::read_toml_file;
use libcnb_data::buildpack::BuildpackDescriptor;
use std::path::Path;

#[must_use]
pub fn determine_buildpack_kind(buildpack_dir: &Path) -> Option<BuildpackKind> {
    read_toml_file::<BuildpackDescriptor>(buildpack_dir.join("buildpack.toml"))
        .ok()
        .map(|buildpack_descriptor| match buildpack_descriptor {
            BuildpackDescriptor::Single(_) => {
                if buildpack_dir.join("Cargo.toml").is_file() {
                    BuildpackKind::LibCnbRs
                } else {
                    BuildpackKind::Other
                }
            }
            BuildpackDescriptor::Meta(_) => BuildpackKind::Meta,
        })
}

pub enum BuildpackKind {
    LibCnbRs,
    Meta,
    Other,
}
