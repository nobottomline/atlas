use serde::{Deserialize, Serialize};

/// A declared capability that a source module requires from the host.
///
/// Sources must declare all capabilities they use in their manifest.
/// The host enforces that a source only calls host-import functions
/// that correspond to its declared capability set.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Make outbound HTTP/HTTPS requests through the host.
    NetworkFetch,
    /// Read cookies scoped to this source's declared domains.
    NetworkCookiesRead,
    /// Write cookies scoped to this source's declared domains.
    NetworkCookiesWrite,
    /// Parse HTML using host-provided facilities.
    HtmlParse,
    /// Read from the host-managed response cache.
    CacheRead,
    /// Write to the host-managed response cache.
    CacheWrite,
    /// Read source-level user preferences.
    PreferencesRead,
    /// Write source-level user preferences.
    PreferencesWrite,
    /// Maintain an authenticated session (login/logout support).
    AuthSession,
    /// Query current wall-clock time.
    TimeNow,
    /// Use cryptographic hash functions.
    CryptoHash,
    /// Emit debug log messages visible to the host diagnostics layer.
    LogDebug,
}

impl Capability {
    /// Canonical string identifier used in manifests and policy logs.
    pub fn identifier(&self) -> &'static str {
        match self {
            Self::NetworkFetch       => "network.fetch",
            Self::NetworkCookiesRead => "network.cookies.read",
            Self::NetworkCookiesWrite => "network.cookies.write",
            Self::HtmlParse          => "html.parse",
            Self::CacheRead          => "cache.read",
            Self::CacheWrite         => "cache.write",
            Self::PreferencesRead    => "preferences.read",
            Self::PreferencesWrite   => "preferences.write",
            Self::AuthSession        => "auth.session",
            Self::TimeNow            => "time.now",
            Self::CryptoHash         => "crypto.hash",
            Self::LogDebug           => "log.debug",
        }
    }

    /// The minimum set of capabilities required by every source.
    pub fn required() -> &'static [Capability] {
        &[Capability::LogDebug]
    }
}

impl core::fmt::Display for Capability {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.identifier())
    }
}
