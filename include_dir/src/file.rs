use std::{
    fmt::{self, Debug, Formatter},
    path::Path,
};

#[cfg(debug_assertions)]
use std::{sync::Mutex, collections::HashMap};
#[cfg(debug_assertions)]
use once_cell::sync::Lazy;

/// In debug mode, the file is not read when compiling, it is read when it is used, and then placed in this cache.
#[cfg(debug_assertions)]
static FILES_CACHE: Lazy<Mutex<HashMap<&'static str, &'static [u8]>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// A file with its contents stored in a `&'static [u8]`.
#[derive(Clone, PartialEq, Eq)]
pub struct File<'a> {
    path: &'a str,
    contents: &'a [u8],
    #[cfg(feature = "metadata")]
    metadata: Option<crate::Metadata>,
    #[cfg(debug_assertions)]
    prefix: &'a str,
}

impl<'a> File<'a> {
    /// Create a new [`File`].
    pub const fn new(path: &'a str, contents: &'a [u8],
                     #[cfg(debug_assertions)]
                     prefix: &'a str,
    ) -> Self {
        File {
            path,
            contents,
            #[cfg(feature = "metadata")]
            metadata: None,
            #[cfg(debug_assertions)]
            prefix,
        }
    }

    /// The full path for this [`File`], relative to the directory passed to
    /// [`crate::include_dir!()`].
    pub fn path(&self) -> &'a Path {
        Path::new(self.path)
    }

    /// The file's raw contents.
    pub fn contents(&self) -> &[u8] {
        #[cfg(debug_assertions)]
        {
            let mut cache = FILES_CACHE.lock().unwrap();
            if !cache.contains_key(self.path) {
                let real_path = self.prefix.to_string().clone() + std::path::MAIN_SEPARATOR.to_string().as_str() + self.path;
                let real_path : &Path = Path::new(real_path.as_str());
                let value = Box::leak(std::fs::read(real_path).unwrap().into_boxed_slice());
                let key = Box::leak(self.path.to_string().into_boxed_str());
                cache.insert(key, value);
            }
            cache.get(self.path).unwrap()
        }
        #[cfg(not(debug_assertions))]
        {
            self.contents
        }
    }

    /// The file's contents interpreted as a string.
    pub fn contents_utf8(&self) -> Option<&str> {
        std::str::from_utf8(self.contents()).ok()
    }
}

#[cfg(feature = "metadata")]
impl<'a> File<'a> {
    /// Set the [`Metadata`] associated with a [`File`].
    pub const fn with_metadata(self, metadata: crate::Metadata) -> Self {
        #[cfg(not(debug_assertions))]
        let File { path, contents , .. } = self;
        #[cfg(debug_assertions)]
        let File { path, contents,prefix , .. } = self;

        File {
            path,
            contents,
            metadata: Some(metadata),
            #[cfg(debug_assertions)]
            prefix,
        }
    }

    /// Get the [`File`]'s [`Metadata`], if available.
    pub fn metadata(&self) -> Option<&crate::Metadata> {
        self.metadata.as_ref()
    }
}

impl<'a> Debug for File<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let File {
            path,
            contents,
            #[cfg(feature = "metadata")]
            metadata,
            #[cfg(debug_assertions)]
            prefix,
        } = self;

        let mut d = f.debug_struct("File");

        d.field("path", path)
            .field("contents", &format!("<{} bytes>", contents.len()));

        #[cfg(feature = "metadata")]
        d.field("metadata", metadata);

        #[cfg(debug_assertions)]
        d.field("prefix", prefix);

        d.finish()
    }
}
