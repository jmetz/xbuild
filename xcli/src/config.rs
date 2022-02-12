use crate::android::AndroidSdk;
use crate::{Format, Platform};
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use xapk::{AndroidManifest, VersionCode};

#[derive(Clone, Debug)]
pub struct Config {
    pub name: String,
    version: String,
    description: String,
    generic: GenericConfig,
    apk: ApkConfig,
    appimage: AppimageConfig,
    msix: MsixConfig,
}

impl Config {
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file_name = path
            .as_ref()
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let contents = std::fs::read_to_string(path.as_ref())?;
        let config = match file_name {
            "Cargo.toml" => {
                let toml: CargoToml = toml::from_str(&contents)?;
                let config = toml
                    .package
                    .metadata
                    .unwrap_or_default()
                    .x
                    .unwrap_or_default();
                Config {
                    name: toml.package.name,
                    version: toml.package.version,
                    description: toml.package.description.unwrap_or_default(),
                    generic: config.generic.unwrap_or_default(),
                    apk: config.apk.unwrap_or_default(),
                    appimage: config.appimage.unwrap_or_default(),
                    msix: config.msix.unwrap_or_default(),
                }
            }
            "pubspec.yaml" => {
                let yaml: Pubspec = serde_yaml::from_str(&contents)?;
                let config = yaml.x.unwrap_or_default();
                Config {
                    name: yaml.name,
                    version: yaml.version,
                    description: yaml.description.unwrap_or_default(),
                    generic: config.generic.unwrap_or_default(),
                    apk: config.apk.unwrap_or_default(),
                    appimage: config.appimage.unwrap_or_default(),
                    msix: config.msix.unwrap_or_default(),
                }
            }
            _ => anyhow::bail!("unsupported config file: {}", file_name),
        };
        Ok(config)
    }

    pub fn icon(&self, format: Format) -> Option<&Path> {
        let icon = match format {
            Format::Apk => self.apk.generic.icon.as_deref(),
            Format::Appimage => self.appimage.generic.icon.as_deref(),
            Format::Msix => self.msix.generic.icon.as_deref(),
            _ => return self.generic.icon.as_deref(),
        };
        if let Some(icon) = icon {
            return Some(icon);
        }
        self.generic.icon.as_deref()
    }

    pub fn target_file(&self, platform: Platform) -> PathBuf {
        let file = Path::new("lib").join(format!("{}.dart", platform));
        if file.exists() {
            file
        } else {
            Path::new("lib").join("main.dart")
        }
    }

    pub fn android_manifest(&self, sdk: &AndroidSdk) -> Result<AndroidManifest> {
        let mut manifest = self.apk.manifest.clone();
        manifest.version_name = Some(self.version.clone());
        manifest.version_code = Some(VersionCode::from_semver(&self.version)?.to_code(1));
        manifest
            .sdk
            .target_sdk_version
            .get_or_insert_with(|| sdk.default_target_platform());
        Ok(manifest)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Pubspec {
    name: String,
    version: String,
    description: Option<String>,
    x: Option<RawConfig>,
}

#[derive(Debug, Clone, Deserialize)]
struct CargoToml {
    package: Package,
}

#[derive(Debug, Clone, Deserialize)]
struct Package {
    name: String,
    version: String,
    description: Option<String>,
    metadata: Option<Metadata>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct Metadata {
    x: Option<RawConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct RawConfig {
    #[serde(flatten)]
    generic: Option<GenericConfig>,
    pub apk: Option<ApkConfig>,
    pub appimage: Option<AppimageConfig>,
    pub msix: Option<MsixConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct GenericConfig {
    icon: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ApkConfig {
    #[serde(flatten)]
    generic: GenericConfig,
    manifest: AndroidManifest,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AppimageConfig {
    #[serde(flatten)]
    generic: GenericConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct MsixConfig {
    #[serde(flatten)]
    generic: GenericConfig,
}
