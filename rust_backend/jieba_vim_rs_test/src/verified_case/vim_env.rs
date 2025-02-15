//! Environment variables related to vim.

use std::env;

/// The vim distribution path or name in PATH.
const VIM_DISTRO_ENV: &str = "VIM_BIN_NAME";
/// The vim bundle path (i.e. plugin path).
const VIM_BUNDLE_ENV: &str = "VIM_BUNDLE_PATH";

/// Vim distribution. The enclosed string is the path or the executable name
/// in PATH.
#[derive(Debug, PartialEq, Eq)]
pub enum VimDistro {
    /// Vim.
    Vim(String),
    /// Neovim.
    Nvim(String),
}

/// Get path base name. Since we only consider verification on Ubuntu
/// currently, it doesn't matter to not considering backslash file path
/// separator. May change in the future.
fn get_base_name(path: &str) -> &str {
    path.rsplit_once('/').map(|(_, name)| name).unwrap_or(path)
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidEnvValue(String);

impl TryFrom<String> for VimDistro {
    type Error = InvalidEnvValue;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let name = get_base_name(&value);
        match name {
            "vim" => Ok(VimDistro::Vim(value)),
            "nvim" => Ok(VimDistro::Nvim(value)),
            _ => Err(InvalidEnvValue(value)),
        }
    }
}

impl VimDistro {
    pub fn new_from_env() -> Self {
        VimDistro::try_from(env::var(VIM_DISTRO_ENV).unwrap_or("vim".into()))
            .unwrap_or_else(|InvalidEnvValue(value)| {
                panic!("Unexpected VIM_BIN_NAME: {}", value)
            })
    }
}

impl AsRef<str> for VimDistro {
    fn as_ref(&self) -> &str {
        match self {
            Self::Vim(value) => value,
            Self::Nvim(value) => value,
        }
    }
}

pub struct VimBundlePath(String);

impl VimBundlePath {
    pub fn new_from_env() -> Self {
        Self(env::var(VIM_BUNDLE_ENV).unwrap_or("~/.vim/bundle".into()))
    }

    /// Get path to vader.vim.
    pub fn get_vader_rtp(&self) -> String {
        match self.0.ends_with('/') {
            false => format!("{}/vader.vim", self.0),
            true => format!("{}vader.vim", self.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_distro_try_from() {
        assert_eq!(
            VimDistro::try_from("nvim".to_string()),
            Ok(VimDistro::Nvim("nvim".into()))
        );
        assert_eq!(
            VimDistro::try_from("/path/to/vim".to_string()),
            Ok(VimDistro::Vim("/path/to/vim".into()))
        );
    }
}
