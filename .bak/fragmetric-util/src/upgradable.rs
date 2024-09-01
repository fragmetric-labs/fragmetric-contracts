#[cfg(feature = "derive")]
pub use fragmetric_util_derive::RequireUpgradable;

#[doc(hidden)]
pub mod __private {
    use super::*;

    /// Alternative [AsMut] for crate-internal usage.
    ///
    #[cfg_attr(
        feature = "derive",
        doc = " This trait is automatically derived by [RequireUpgradable] macro,"
    )]
    #[cfg_attr(
        not(feature = "derive"),
        doc = " It's only purpose is to support [Upgradable] trait,"
    )]
    /// so you **MUST NOT** implement or use this trait directly.
    pub trait __AsMut<T> {
        #[cfg_attr(
            feature = "derive",
            doc = " This trait is automatically derived by [RequireUpgradable] macro,"
        )]
        #[cfg_attr(
            not(feature = "derive"),
            doc = " It's only purpose is to support [Upgradable] trait,"
        )]
        /// so you **MUST NOT** implement or use this trait directly.
        fn __as_mut(&mut self) -> &mut T;
    }

    /// Trait derived by macro.
    pub trait __RequireUpgradable<T>: Upgradable<LatestVersion = T> {}
}

/// An upgradable versioned data account.
pub trait Upgradable: __private::__AsMut<Self::LatestVersion> {
    /// Latest version type.
    type LatestVersion;

    /// Upgrade the upgradable data field into latest version.
    ///
    /// You must properly upgrade into [LatestVersion],
    /// otherwise [`to_latest_version`] method will panic,
    /// since it will dereference the type into [LatestVersion] after upgrade.
    ///
    /// [LatestVersion]: Upgradable::LatestVersion
    /// [`to_latest_version`]: Upgradable::to_latest_version
    fn upgrade(&mut self);

    /// Unwraps the upgradable data field into latest version,
    /// assuming that it's internal type is [LatestVersion] when upgraded.
    ///
    /// ## Panic
    ///
    /// Panics unless [`upgrade`] method does not properly upgrade into [LatestVersion].
    ///
    /// [LatestVersion]: Upgradable::LatestVersion
    /// [`upgrade`]: Upgradable::upgrade
    fn to_latest_version(&mut self) -> &mut Self::LatestVersion {
        self.upgrade();
        self.__as_mut()
    }
}
