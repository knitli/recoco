pub mod shared;

#[cfg(feature = "source-s3")]
pub mod amazon_s3;
#[cfg(feature = "source-azure")]
pub mod azure_blob;
#[cfg(feature = "source-gdrive")]
pub mod google_drive;
#[cfg(feature = "source-local-file")]
pub mod local_file;
#[cfg(feature = "source-postgres")]
pub mod postgres;
