use std::borrow::Cow;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::PoisonError;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

use crate::Result;

macro_rules! env_error {
    ($($arg:tt)*) => {
        $crate::Error::env(format!($($arg)*))
    };
}

/// Abstract environment interface.
///
/// Provides access to command-line arguments, environment variables,
/// and well-known directories (current, home, temp, executable).
/// Changes are scoped to the platform — no global process state is
/// modified — and take effect for its fs, process and request: they
/// resolve relative paths against the current directory, child
/// processes get exactly its variables, and the proxies of the default
/// clients use them.
///
/// Arguments, keys and values are [`OsString`]s as given by the OS, none
/// is dropped or converted lossily. Convert them explicitly, e.g. with
/// [`OsString::into_string`] or [`OsStr::to_string_lossy`].
///
/// The environment can be changed through a shared reference, so a
/// platform can be shared between threads and tasks, e.g. in an
/// [`Arc`]. Like with [`std::env`](mod@std::env) a change while other
/// threads use the platform takes effect for their next operation.
///
/// The two provided implementations are `EnvImpl` (each value loaded from
/// [`std::env`](mod@std::env) on first use) and [`TestEnv`](crate::test_platform::test_env::TestEnv)
/// (purely in-memory, behind the `testing` feature).
pub trait Env: Send + Sync {
    /// Returns an iterator over the command-line arguments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Env, Platform};
    ///
    /// let platform = agplatform::platform();
    /// platform.env().args().for_each(|arg| {
    ///     println!("{}", arg.display());
    /// });
    /// ```
    fn args(&self) -> EnvArgs<'_>;

    /// Returns the current directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Env, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let current_dir = platform.env().current_dir();
    /// assert_eq!(current_dir, std::env::current_dir().unwrap());
    /// ```
    fn current_dir(&self) -> PathBuf;

    /// Returns a reference to the current executable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Env, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let current_exe = platform.env().current_exe();
    /// assert_eq!(current_exe, std::env::current_exe().unwrap());
    /// ```
    fn current_exe(&self) -> &Path;

    /// Returns the home directory: the `HOME` variable on Unix,
    /// `USERPROFILE` on Windows, `None` when it is not set or empty. Unlike
    /// [`std::env::home_dir`] it does not fall back to the user database.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Env, Platform};
    ///
    /// let platform = agplatform::platform();
    /// if let Some(home_dir) = platform.env().home_dir() {
    ///     println!("home: {}", home_dir.display());
    /// }
    /// ```
    fn home_dir(&self) -> Option<PathBuf>;

    /// Removes the environment variable with the given key and returns
    /// its value if it existed. On Windows the key is matched ASCII case
    /// insensitively like in the Windows environment.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Env, Platform};
    ///
    /// let platform = agplatform::platform();
    /// platform.env().set_var("TEST_VAR", "value");
    /// let removed_var = platform.env().remove_var("TEST_VAR");
    /// assert_eq!(removed_var, Some("value".into()));
    /// ```
    fn remove_var<K: AsRef<OsStr>>(&self, key: K) -> Option<OsString>;

    /// Sets the current directory and returns the previous value. A
    /// relative `path` is resolved against the previous current directory
    /// and the `.` and `..` components are removed lexically, never going
    /// above the root.
    ///
    /// # Errors
    ///
    /// Returns an error, leaving the current directory unchanged, if the
    /// resolved path is not an existing directory, like
    /// [`std::env::set_current_dir`]. Symlinks are followed. A
    /// [`TestEnv`](crate::TestEnv) does not check it. Also returns an error
    /// for a relative `path` if the current directory is not known, e.g.
    /// because the OS could not tell it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Env, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let dir = agplatform::TestDir::new();
    /// std::fs::create_dir(dir.path().join("a")).unwrap();
    ///
    /// let old_dir = platform.env().set_current_dir(dir.path().join("a")).unwrap();
    /// assert_eq!(old_dir, std::env::current_dir().unwrap());
    /// platform.env().set_current_dir("..").unwrap();
    /// assert_eq!(platform.env().current_dir(), dir.path());
    /// assert!(platform.env().set_current_dir("missing/dir").is_err());
    /// ```
    fn set_current_dir<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf>;

    /// Sets the environment variable with the given key and value,
    /// returning the previous value if it existed. On Windows the key is
    /// matched ASCII case insensitively like in the Windows environment
    /// and an existing variable keeps the spelling of its key.
    ///
    /// # Errors
    ///
    /// Returns an error, leaving the variables unchanged, if the key is
    /// empty or contains `=` or NUL, or if the value contains NUL. The OS
    /// cannot pass such a variable to a child process. Where
    /// [`std::env::set_var`] panics, this returns the error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Env, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let old_var = platform.env().set_var("TEST_VAR", "value").unwrap();
    /// assert_eq!(old_var, None);
    /// let var = platform.env().var("TEST_VAR");
    /// assert_eq!(var, Some("value".into()));
    /// assert!(platform.env().set_var("A=B", "value").is_err());
    /// ```
    fn set_var<K: Into<OsString>, V: Into<OsString>>(
        &self,
        key: K,
        value: V,
    ) -> Result<Option<OsString>>;

    /// Returns the temporary directory like [`std::env::temp_dir`]: the
    /// `TMPDIR` variable on Unix, otherwise `/tmp` (the per-user temporary
    /// directory on macOS and iOS, `/data/local/tmp` on Android). On Windows
    /// the first set of `TMP`, `TEMP` and `USERPROFILE`, the Windows
    /// directory (`SystemRoot`) otherwise, ending with a separator. Unlike
    /// std, a process running as SYSTEM gets these too, not
    /// `C:\Windows\SystemTemp`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Env, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let tmp_dir = platform.env().tmp_dir();
    /// assert_eq!(tmp_dir, std::env::temp_dir());
    /// ```
    fn tmp_dir(&self) -> PathBuf;

    /// Returns the value of the environment variable with the given
    /// key, if it exists. On Windows the key is matched ASCII case
    /// insensitively like in the Windows environment.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Env, Platform};
    ///
    /// let platform = agplatform::platform();
    /// if let Some(path) = platform.env().var("PATH") {
    ///     println!("PATH: {}", path.display());
    /// }
    /// ```
    fn var<K: AsRef<OsStr>>(&self, key: K) -> Option<OsString>;

    /// Returns the boolean value of the environment variable with the given key.
    /// Interprets case insensitively "true", "on", "1" as `true` and "false",
    /// "off", "0", empty, or missing as `false`. The value is trimmed and then
    /// one pair of surrounding quotes is removed, the inside of the quotes is
    /// not trimmed (e.g. `' true '` is invalid). Returns an error if the value
    /// cannot be interpreted as a boolean.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The value is present but cannot be interpreted as a boolean
    ///   (e.g. `"invalid"`, `"2"` or a value that is not valid UTF-8).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Env, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let value = platform.env().var_bool("TEST_BOOL_MISSING").unwrap();
    /// assert!(!value);
    ///
    /// platform.env().set_var("TEST_BOOL_TRUE", "true").unwrap();
    /// let value = platform.env().var_bool("TEST_BOOL_TRUE").unwrap();
    /// assert!(value);
    ///
    /// platform.env().set_var("TEST_BOOL_INVALID", "invalid").unwrap();
    /// let result = platform.env().var_bool("TEST_BOOL_INVALID");
    /// assert!(result.is_err());
    /// ```
    fn var_bool<K: AsRef<OsStr>>(&self, key: K) -> Result<bool> {
        let key = key.as_ref();

        let Some(value) = self.var(key) else {
            return Ok(false);
        };

        // WHY: a value that is not valid UTF-8 gets a replacement character,
        // which no boolean matches.
        match crate::utils::unquote(&value.to_string_lossy().to_lowercase()) {
            "true" | "on" | "1" => Ok(true),
            "" | "false" | "off" | "0" => Ok(false),
            _ => Err(env_error!(
                "Invalid boolean value for key '{}': '{}' (expected [any case]: true, on, 1 / false, off, 0, <empty>, <missing>)",
                key.display(),
                value.display()
            )),
        }
    }

    /// Returns an iterator over a snapshot of all environment variables as
    /// key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Env, Platform};
    ///
    /// let platform = agplatform::platform();
    /// platform.env().vars().for_each(|(key, value)| {
    ///     println!("{}: {}", key.display(), value.display());
    /// });
    /// ```
    fn vars(&self) -> EnvVars;
}

/// In-process [`Env`] implementation backed by owned data.
///
/// Each value is loaded from [`std::env`](mod@std::env) on first use, if at
/// all, via [`new_from_std`](Self::new_from_std), and then lives in memory
/// for the lifetime of the instance. Changes (e.g. [`set_var`](Env::set_var),
/// [`set_current_dir`](Env::set_current_dir)) affect only this instance, its
/// clones and the platform it belongs to — the global process environment is
/// never modified.
///
/// The arguments, variables, current directory and current executable are
/// each loaded separately, so they do not form one snapshot of the process.
/// Changes of the process environment after a value was loaded (e.g. by a C
/// library or an unsafe [`std::env::set_var`]) are not seen by it. While the
/// variables are unchanged, child processes inherit the live process
/// environment, which then may differ from [`vars`](Env::vars).
///
/// Of variables the OS lists more than once (possible on Unix) the first is
/// kept, like by [`std::env::var_os`].
///
/// Clones share the environment, e.g. the env of a platform with its fs,
/// process and request so that they see its changes.
///
/// Normally obtained via [`platform()`](crate::platform) rather than
/// constructed directly.
///
/// # Examples
///
/// ```rust
/// use agplatform::{Env, Platform};
///
/// let platform = agplatform::platform();
/// let current_dir = platform.env().current_dir();
/// assert_eq!(current_dir, std::env::current_dir().unwrap());
///
/// platform.env().set_var("TEST_VAR", "value").unwrap();
/// let var = platform.env().var("TEST_VAR");
/// assert_eq!(var, Some("value".into()));
/// ```
#[derive(Clone)]
pub struct EnvImpl(Arc<EnvState>);

/// An iterator over the variables of an [`Env`], see [`Env::vars`].
///
/// Created from a [`Vec`] of key-value pairs or by collecting them.
#[derive(Clone, Debug, Default)]
pub struct EnvVars(std::vec::IntoIter<(OsString, OsString)>);

/// An iterator over the arguments of an [`Env`], see [`Env::args`].
///
/// Created from a slice of the arguments.
#[derive(Clone, Debug, Default)]
pub struct EnvArgs<'a>(std::slice::Iter<'a, OsString>);

/// [internal]
struct EnvState {
    args: OnceLock<Vec<OsString>>,
    current_dir: OnceLock<RwLock<Arc<Path>>>,
    current_exe: OnceLock<PathBuf>,
    vars: OnceLock<RwLock<Vars>>,
}

/// [internal]
struct Vars {
    list: Vec<(OsString, OsString)>,
    /// Incremented by each change, 0 only while these are still the
    /// variables of the current process.
    generation: u64,
}

impl EnvImpl {
    /// Creates an env with no arguments, an empty current executable and
    /// the given variables and current directory.
    #[cfg(any(test, feature = "testing", feature = "request"))]
    pub(crate) fn new(vars: Vec<(OsString, OsString)>, current_dir: PathBuf) -> Self {
        Self(Arc::new(EnvState {
            args: OnceLock::from(Vec::new()),
            current_dir: OnceLock::from(RwLock::new(current_dir.into())),
            current_exe: OnceLock::from(PathBuf::new()),
            vars: OnceLock::from(RwLock::new(Vars {
                list: last_of_each_key(vars),
                generation: 1,
            })),
        }))
    }

    /// Creates empty lazily initialized EnvImpl. It does not read anything
    /// from the OS.
    pub fn new_from_std() -> Self {
        Self(Arc::new(EnvState {
            args: OnceLock::new(),
            current_dir: OnceLock::new(),
            current_exe: OnceLock::new(),
            vars: OnceLock::new(),
        }))
    }

    /// Returns the current directory without copying it.
    pub(crate) fn current_dir_arc(&self) -> Arc<Path> {
        read(self.current_dir_lock()).clone()
    }

    /// Replaces the arguments of an env that is not shared yet.
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn replace_args(&mut self, args: Vec<OsString>) {
        self.state_mut().args = OnceLock::from(args);
    }

    /// Replaces the current directory by `current_dir` as is.
    ///
    /// WHY: a root, not a full absolute path, so that test paths like
    /// `/home` work on Windows too.
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn replace_current_dir(&self, current_dir: PathBuf) {
        debug_assert!(
            current_dir.has_root(),
            "the current directory should have a root"
        );
        *write(self.current_dir_lock()) = current_dir.into();
    }

    /// Replaces the current executable of an env that is not shared yet.
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn replace_current_exe(&mut self, current_exe: PathBuf) {
        self.state_mut().current_exe = OnceLock::from(current_exe);
    }

    /// Sets the variables to `vars`, the last of those with the same key
    /// kept, e.g. to seed a test env. Panics in debug builds if a variable
    /// is invalid, see [`Env::set_var`].
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn replace_vars(&self, vars: Vec<(OsString, OsString)>) {
        #[cfg(debug_assertions)]
        for (key, value) in &vars {
            if let Err(error) = check_var(key, value) {
                panic!("{error}");
            }
        }

        let mut state = write(self.vars_lock());
        state.list = last_of_each_key(vars);
        state.generation += 1;
    }

    /// Sets the current directory to `path` resolved against the current
    /// one if `check` accepts it, see [`Env::set_current_dir`], and returns
    /// the previous one.
    ///
    /// WHY: checked without the lock so that a slow check (e.g. on a network
    /// drive) does not block the users of the current directory, and done
    /// again if the current directory changed meanwhile.
    pub(crate) fn set_current_dir_checked(
        &self,
        path: &Path,
        check: &dyn Fn(&Path) -> std::io::Result<()>,
    ) -> Result<PathBuf> {
        loop {
            let current_dir = self.current_dir_arc();

            if path.is_relative() && current_dir.as_os_str().is_empty() {
                return Err(env_error!(
                    "Current directory could not be set, the current directory is not known: {}",
                    path.display()
                ));
            }

            let new_dir = crate::utils::resolve_path(&current_dir, path);

            check(&new_dir).map_err(|error| {
                env_error!(
                    "Current directory could not be set, {error}: {}",
                    new_dir.display()
                )
                .with_source(error)
            })?;

            let mut locked = write(self.current_dir_lock());

            if Arc::ptr_eq(&locked, &current_dir) {
                return Ok(std::mem::replace(&mut *locked, new_dir.into()).to_path_buf());
            }
        }
    }

    /// Returns the variables unless they are still those of the current
    /// process, e.g. for a child process to inherit them instead.
    #[cfg(feature = "process")]
    pub(crate) fn changed_vars(&self) -> Option<Vec<(OsString, OsString)>> {
        let vars = read(self.0.vars.get()?);
        (vars.generation != 0).then(|| vars.list.clone())
    }

    /// Returns the result of `f` called with the generation of the
    /// variables, incremented by each change, and the variables.
    #[cfg(feature = "request")]
    pub(crate) fn with_vars<R>(&self, f: impl FnOnce(u64, &[(OsString, OsString)]) -> R) -> R {
        let vars = read(self.vars_lock());
        f(vars.generation, &vars.list)
    }

    fn current_dir_lock(&self) -> &RwLock<Arc<Path>> {
        self.0
            .current_dir
            .get_or_init(|| RwLock::new(std::env::current_dir().unwrap_or_default().into()))
    }

    #[cfg(any(test, feature = "testing"))]
    fn state_mut(&mut self) -> &mut EnvState {
        Arc::get_mut(&mut self.0).expect("an env is not shared while it is being built")
    }

    fn vars_lock(&self) -> &RwLock<Vars> {
        self.0.vars.get_or_init(|| {
            RwLock::new(Vars {
                list: first_of_each_key(std::env::vars_os().collect()),
                generation: 0,
            })
        })
    }
}

fn read<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(PoisonError::into_inner)
}

fn write<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write().unwrap_or_else(PoisonError::into_inner)
}

/// Returns `true` if the variable keys `a` and `b` are the same, ignoring
/// the ASCII case on Windows like its environment.
pub(crate) fn is_same_key(a: &OsStr, b: &OsStr) -> bool {
    #[cfg(windows)]
    return a.eq_ignore_ascii_case(b);
    #[cfg(not(windows))]
    return a == b;
}

/// Returns a value that is equal for all keys that are the same key, see
/// [`is_same_key`].
fn key_id(key: &OsStr) -> Cow<'_, OsStr> {
    #[cfg(windows)]
    return Cow::Owned(key.to_ascii_uppercase());
    #[cfg(not(windows))]
    return Cow::Borrowed(key);
}

/// Returns `vars` without those whose key was already seen.
fn first_of_each_key(vars: Vec<(OsString, OsString)>) -> Vec<(OsString, OsString)> {
    let mut seen = HashSet::new();
    let first: Vec<bool> = vars
        .iter()
        .map(|(key, _)| seen.insert(key_id(key)))
        .collect();
    drop(seen);

    vars.into_iter()
        .zip(first)
        .filter_map(|(var, first)| first.then_some(var))
        .collect()
}

/// Returns `vars` without those whose key is seen again later.
#[cfg(any(test, feature = "testing", feature = "request"))]
fn last_of_each_key(mut vars: Vec<(OsString, OsString)>) -> Vec<(OsString, OsString)> {
    vars.reverse();
    let mut vars = first_of_each_key(vars);
    vars.reverse();
    vars
}

/// Fails unless the variable can be passed to a child process, see
/// [`Env::set_var`].
fn check_var(key: &OsStr, value: &OsStr) -> Result<()> {
    let key_bytes = key.as_encoded_bytes();

    if key_bytes.is_empty() || key_bytes.contains(&b'=') || key_bytes.contains(&0) {
        return Err(env_error!(
            "Invalid environment variable key '{}': it must not be empty or contain '=' or NUL",
            key.display()
        ));
    }

    if value.as_encoded_bytes().contains(&0) {
        return Err(env_error!(
            "Invalid value of environment variable '{}': it must not contain NUL",
            key.display()
        ));
    }

    Ok(())
}

/// Fails unless `path` is an existing directory, following symlinks, see
/// [`Env::set_current_dir`].
fn is_dir(path: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    if is_plain_dir(path) {
        return Ok(());
    }

    if std::fs::metadata(path)?.is_dir() {
        Ok(())
    } else {
        Err(std::io::ErrorKind::NotADirectory.into())
    }
}

/// Returns `true` if `path` is a directory that is not a reparse point,
/// e.g. a symlink.
///
/// WHY: one call, while [`std::fs::metadata`] opens the directory, which
/// makes it many times slower than [`std::env::set_current_dir`].
#[cfg(windows)]
fn is_plain_dir(path: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt;

    const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileAttributesW(name: *const u16) -> u32;
    }

    let name: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let mask = FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT;

    // WHY: a NUL inside the path would cut it short. INVALID_FILE_ATTRIBUTES
    // has all bits set, so errors are not plain directories either.
    name.iter().position(|&unit| unit == 0) == Some(name.len() - 1)
        // SAFETY: `name` is NUL terminated.
        && unsafe { GetFileAttributesW(name.as_ptr()) } & mask == FILE_ATTRIBUTE_DIRECTORY
}

/// The temporary directory when no variable sets it, see [`Env::tmp_dir`].
#[cfg(all(unix, not(target_os = "android"), not(target_vendor = "apple")))]
const DEFAULT_TMP_DIR: &str = "/tmp";
/// The temporary directory when no variable sets it, see [`Env::tmp_dir`].
#[cfg(target_os = "android")]
const DEFAULT_TMP_DIR: &str = "/data/local/tmp";
/// The Windows directory when `SystemRoot` is not set, see [`Env::tmp_dir`].
#[cfg(windows)]
const DEFAULT_TMP_DIR: &str = r"C:\Windows";

#[cfg(not(any(unix, windows)))]
compile_error!("agplatform supports only Unix and Windows targets");

/// The temporary directory of the user when no variable sets it, `/tmp`
/// if it cannot be told, like by [`std::env::temp_dir`], see
/// [`Env::tmp_dir`].
#[cfg(target_vendor = "apple")]
fn default_tmp_dir() -> PathBuf {
    use std::os::unix::ffi::OsStringExt;

    const _CS_DARWIN_USER_TEMP_DIR: std::ffi::c_int = 65537;

    unsafe extern "C" {
        fn confstr(name: std::ffi::c_int, buf: *mut std::ffi::c_char, len: usize) -> usize;
    }

    let mut buf = vec![0_u8; 64];

    loop {
        // SAFETY: `buf` is valid for writes of its length.
        let len = unsafe { confstr(_CS_DARWIN_USER_TEMP_DIR, buf.as_mut_ptr().cast(), buf.len()) };

        if len == 0 {
            return PathBuf::from("/tmp");
        }

        // WHY: `len` includes the terminating NUL.
        if len <= buf.len() {
            buf.truncate(len - 1);
            return PathBuf::from(OsString::from_vec(buf));
        }

        buf.resize(len, 0);
    }
}

/// The temporary directory when no variable sets it, see [`Env::tmp_dir`].
#[cfg(not(target_vendor = "apple"))]
fn default_tmp_dir() -> PathBuf {
    PathBuf::from(DEFAULT_TMP_DIR)
}

/// The variable of the home directory, see [`Env::home_dir`].
#[cfg(unix)]
const HOME_VAR: &str = "HOME";
/// The variable of the home directory, see [`Env::home_dir`].
#[cfg(windows)]
const HOME_VAR: &str = "USERPROFILE";

impl Env for EnvImpl {
    fn args(&self) -> EnvArgs<'_> {
        EnvArgs::from(
            self.0
                .args
                .get_or_init(|| std::env::args_os().collect())
                .as_slice(),
        )
    }

    fn current_dir(&self) -> PathBuf {
        self.current_dir_arc().to_path_buf()
    }

    fn current_exe(&self) -> &Path {
        self.0
            .current_exe
            .get_or_init(|| std::env::current_exe().unwrap_or_default())
    }

    fn home_dir(&self) -> Option<PathBuf> {
        self.var(HOME_VAR)
            .filter(|home| !home.is_empty())
            .map(PathBuf::from)
    }

    fn remove_var<K: AsRef<OsStr>>(&self, key: K) -> Option<OsString> {
        let mut vars = write(self.vars_lock());
        let pos = vars
            .list
            .iter()
            .position(|(k, _)| is_same_key(k, key.as_ref()))?;
        vars.generation += 1;
        Some(vars.list.remove(pos).1)
    }

    fn set_current_dir<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        self.set_current_dir_checked(path.as_ref(), &is_dir)
    }

    fn set_var<K: Into<OsString>, V: Into<OsString>>(
        &self,
        key: K,
        value: V,
    ) -> Result<Option<OsString>> {
        let (key, value) = (key.into(), value.into());
        check_var(&key, &value)?;
        let mut vars = write(self.vars_lock());

        let previous =
            if let Some((_, v)) = vars.list.iter_mut().find(|(k, _)| is_same_key(k, &key)) {
                if *v == value {
                    return Ok(Some(value));
                }

                Some(std::mem::replace(v, value))
            } else {
                vars.list.push((key, value));
                None
            };

        vars.generation += 1;
        Ok(previous)
    }

    #[cfg(unix)]
    fn tmp_dir(&self) -> PathBuf {
        self.var("TMPDIR")
            .map_or_else(default_tmp_dir, PathBuf::from)
    }

    // WHY: like `GetTempPath2W`, which std calls and which ends the path
    // with a separator.
    #[cfg(windows)]
    fn tmp_dir(&self) -> PathBuf {
        let mut tmp_dir = ["TMP", "TEMP", "USERPROFILE", "SystemRoot"]
            .iter()
            .find_map(|key| self.var(key))
            .map_or_else(default_tmp_dir, PathBuf::from);
        tmp_dir.push("");
        tmp_dir
    }

    fn var<K: AsRef<OsStr>>(&self, key: K) -> Option<OsString> {
        read(self.vars_lock())
            .list
            .iter()
            .find(|(k, _)| is_same_key(k, key.as_ref()))
            .map(|(_, v)| v.clone())
    }

    fn vars(&self) -> EnvVars {
        EnvVars::from(read(self.vars_lock()).list.clone())
    }
}

impl<'a> From<&'a [OsString]> for EnvArgs<'a> {
    fn from(args: &'a [OsString]) -> Self {
        Self(args.iter())
    }
}

impl<'a> Iterator for EnvArgs<'a> {
    type Item = &'a OsString;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for EnvArgs<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl ExactSizeIterator for EnvArgs<'_> {}

impl std::iter::FusedIterator for EnvArgs<'_> {}

impl From<Vec<(OsString, OsString)>> for EnvVars {
    fn from(vars: Vec<(OsString, OsString)>) -> Self {
        Self(vars.into_iter())
    }
}

impl FromIterator<(OsString, OsString)> for EnvVars {
    fn from_iter<I: IntoIterator<Item = (OsString, OsString)>>(vars: I) -> Self {
        Self::from(vars.into_iter().collect::<Vec<_>>())
    }
}

impl Iterator for EnvVars {
    type Item = (OsString, OsString);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for EnvVars {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl ExactSizeIterator for EnvVars {}

impl std::iter::FusedIterator for EnvVars {}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// The contract cases of [`Env`], each run on an env without variables
    /// unless it says otherwise.
    pub(crate) mod env {
        use super::*;

        pub(crate) mod args {
            use super::*;

            /// `env` has the arguments `args`.
            pub(crate) fn args_should_return_arguments_in_order(env: &impl Env, args: &[&str]) {
                assert_eq!(env.args().collect::<Vec<_>>(), args);
                assert_eq!(env.args().len(), args.len());
                assert_eq!(
                    env.args().rev().collect::<Vec<_>>(),
                    args.iter().rev().collect::<Vec<_>>()
                );
            }
        }

        pub(crate) mod current_dir {
            use super::*;

            /// `env` is in `expected`.
            pub(crate) fn current_dir_should_return_current_directory(
                env: &impl Env,
                expected: &Path,
            ) {
                assert_eq!(env.current_dir(), expected);
            }
        }

        pub(crate) mod current_exe {
            use super::*;

            /// `env` runs `expected`.
            pub(crate) fn current_exe_should_return_current_executable(
                env: &impl Env,
                expected: &Path,
            ) {
                assert_eq!(env.current_exe(), expected);
            }
        }

        pub(crate) mod home_dir {
            use super::*;

            pub(crate) fn home_dir_should_return_home_variable(env: &impl Env) {
                env.set_var(HOME_VAR, "/home/other").unwrap();

                assert_eq!(env.home_dir(), Some(PathBuf::from("/home/other")));
            }

            pub(crate) fn home_dir_unset_or_empty_variable_should_return_none(env: &impl Env) {
                assert_eq!(env.home_dir(), None);

                env.set_var(HOME_VAR, "").unwrap();

                assert_eq!(env.home_dir(), None);
            }
        }

        pub(crate) mod remove_var {
            use super::*;

            pub(crate) fn remove_var_should_return_value_for_present_key(env: &impl Env) {
                env.set_var("KEY", "value").unwrap();

                assert_eq!(env.remove_var("KEY"), Some("value".into()));
                assert_eq!(env.var("KEY"), None);
            }

            pub(crate) fn remove_var_should_return_none_for_missing_key(env: &impl Env) {
                assert_eq!(env.remove_var("MISSING"), None);
            }

            #[cfg(windows)]
            pub(crate) fn remove_var_should_match_key_ignoring_ascii_case(env: &impl Env) {
                env.set_var("Path", "value").unwrap();

                assert_eq!(env.remove_var("PATH"), Some("value".into()));
                assert_eq!(env.vars().len(), 0);
            }

            #[cfg(unix)]
            pub(crate) fn remove_var_should_match_key_case_sensitively(env: &impl Env) {
                env.set_var("Path", "value").unwrap();

                assert_eq!(env.remove_var("PATH"), None);
                assert_eq!(env.var("Path"), Some("value".into()));
            }
        }

        pub(crate) mod set_current_dir {
            use super::*;

            /// `dir` is an existing absolute directory.
            pub(crate) fn set_current_dir_should_return_previous_value(env: &impl Env, dir: &Path) {
                let previous = env.current_dir();

                assert_eq!(env.set_current_dir(dir).unwrap(), previous);
            }

            /// `dir` is an existing absolute directory.
            pub(crate) fn set_current_dir_should_update_current_directory(
                env: &impl Env,
                dir: &Path,
            ) {
                env.set_current_dir(dir).unwrap();

                assert_eq!(env.current_dir(), dir);
            }

            /// `dir` is an existing absolute directory with the directory `a`.
            pub(crate) fn set_current_dir_relative_should_resolve_against_previous_current_dir(
                env: &impl Env,
                dir: &Path,
            ) {
                env.set_current_dir(dir).unwrap();

                let previous = env.set_current_dir("b/../a/.").unwrap();

                assert_eq!(previous, dir);
                assert_eq!(env.current_dir(), dir.join("a"));
                assert_eq!(env.set_current_dir("..").unwrap(), dir.join("a"));
                assert_eq!(env.current_dir(), dir);
            }

            /// `dir` is an existing absolute directory.
            pub(crate) fn set_current_dir_should_not_go_above_root(env: &impl Env, dir: &Path) {
                env.set_current_dir(dir).unwrap();
                let up = vec![".."; dir.components().count() + 1].join("/");

                env.set_current_dir(&up).unwrap();

                assert_eq!(env.current_dir(), dir.ancestors().last().unwrap());
            }
        }

        pub(crate) mod set_var {
            use super::*;

            pub(crate) fn set_var_should_return_none_for_new_key(env: &impl Env) {
                assert_eq!(env.set_var("KEY", "value").unwrap(), None);
                assert_eq!(env.var("KEY"), Some("value".into()));
            }

            pub(crate) fn set_var_should_return_previous_value_for_existing_key(env: &impl Env) {
                env.set_var("KEY", "value").unwrap();

                assert_eq!(env.set_var("KEY", "value2").unwrap(), Some("value".into()));
                assert_eq!(env.var("KEY"), Some("value2".into()));
            }

            pub(crate) fn set_var_invalid_key_should_fail_and_keep_vars(env: &impl Env) {
                for key in ["", "A=B", "=A", "A\0B"] {
                    let error = env.set_var(key, "value").unwrap_err();

                    assert_eq!(
                        error.description(),
                        format!(
                            "Invalid environment variable key '{key}': it must not be empty or contain '=' or NUL"
                        ),
                        "key: {key:?}"
                    );
                    assert_eq!(env.vars().len(), 0, "key: {key:?}");
                }
            }

            pub(crate) fn set_var_value_with_nul_should_fail_and_keep_vars(env: &impl Env) {
                env.set_var("A", "1").unwrap();

                let error = env.set_var("A", "a\0b").unwrap_err();

                assert_eq!(
                    error.description(),
                    "Invalid value of environment variable 'A': it must not contain NUL"
                );
                assert_eq!(env.var("A"), Some("1".into()));
            }

            #[cfg(windows)]
            pub(crate) fn set_var_different_case_should_replace_value_keeping_key(env: &impl Env) {
                env.set_var("Path", "value").unwrap();

                let previous = env.set_var("PATH", "value2").unwrap();

                assert_eq!(previous, Some("value".into()));
                assert_eq!(
                    env.vars().collect::<Vec<_>>(),
                    [("Path".into(), "value2".into())]
                );
            }

            #[cfg(unix)]
            pub(crate) fn set_var_different_case_should_add_variable(env: &impl Env) {
                env.set_var("Path", "value").unwrap();

                let previous = env.set_var("PATH", "value2").unwrap();

                assert_eq!(previous, None);
                assert_eq!(
                    env.vars().collect::<Vec<_>>(),
                    [
                        ("Path".into(), "value".into()),
                        ("PATH".into(), "value2".into())
                    ]
                );
            }
        }

        pub(crate) mod tmp_dir {
            use super::*;

            #[cfg(unix)]
            pub(crate) fn tmp_dir_should_return_tmpdir_variable(env: &impl Env) {
                env.set_var("TMPDIR", "/var/tmp").unwrap();

                assert_eq!(env.tmp_dir(), PathBuf::from("/var/tmp"));
            }

            #[cfg(unix)]
            pub(crate) fn tmp_dir_unset_variable_should_return_default(env: &impl Env) {
                assert_eq!(env.tmp_dir(), default_tmp_dir());
            }

            #[cfg(windows)]
            pub(crate) fn tmp_dir_should_return_first_set_variable_with_separator(env: &impl Env) {
                for (vars, expected) in [
                    (vec![("TMP", r"C:\tmp"), ("TEMP", r"C:\temp")], r"C:\tmp\"),
                    (
                        vec![("TEMP", r"C:\temp\"), ("USERPROFILE", r"C:\u")],
                        r"C:\temp\",
                    ),
                    (
                        vec![("USERPROFILE", r"C:\u"), ("SystemRoot", r"D:\w")],
                        r"C:\u\",
                    ),
                    (vec![("SystemRoot", r"D:\w")], r"D:\w\"),
                    (vec![], r"C:\Windows\"),
                ] {
                    for key in ["TMP", "TEMP", "USERPROFILE", "SystemRoot"] {
                        env.remove_var(key);
                    }

                    for (key, value) in &vars {
                        env.set_var(*key, *value).unwrap();
                    }

                    assert_eq!(env.tmp_dir(), PathBuf::from(expected), "vars: {vars:?}");
                }
            }
        }

        pub(crate) mod var {
            use super::*;

            pub(crate) fn var_should_return_none_for_missing_key(env: &impl Env) {
                assert_eq!(env.var("MISSING"), None);
            }

            pub(crate) fn var_should_return_value_for_present_key(env: &impl Env) {
                env.set_var("KEY", "value").unwrap();

                assert_eq!(env.var("KEY"), Some("value".into()));
            }

            pub(crate) fn var_should_return_value_that_is_not_utf8(env: &impl Env) {
                env.set_var("KEY", invalid_utf8()).unwrap();

                assert_eq!(env.var("KEY"), Some(invalid_utf8()));
            }

            #[cfg(windows)]
            pub(crate) fn var_should_match_key_ignoring_ascii_case(env: &impl Env) {
                env.set_var("Path", "value").unwrap();

                assert_eq!(env.var("PATH"), Some("value".into()));
            }

            #[cfg(unix)]
            pub(crate) fn var_should_match_key_case_sensitively(env: &impl Env) {
                env.set_var("Path", "value").unwrap();

                assert_eq!(env.var("PATH"), None);
            }
        }

        pub(crate) mod var_bool {
            use super::*;

            pub(crate) fn var_bool_should_accept_truthy_values(env: &impl Env) {
                env.set_var("BOOL_TRUE", "true").unwrap();
                env.set_var("BOOL_TRUE_QUOTED", "\'true\'").unwrap();
                env.set_var("BOOL_TRUE_DOUBLE_QUOTED", "\"true\"").unwrap();
                env.set_var("BOOL_TRUE_UPPER", "TRUE").unwrap();
                env.set_var("BOOL_TRUE_MIXED_CASE", "True").unwrap();
                env.set_var("BOOL_ON", "on").unwrap();
                env.set_var("BOOL_ON_QUOTED", "\'on\'").unwrap();
                env.set_var("BOOL_ON_DOUBLE_QUOTED", "\"on\"").unwrap();
                env.set_var("BOOL_ON_UPPER", "ON").unwrap();
                env.set_var("BOOL_ON_MIXED_CASE", "On").unwrap();
                env.set_var("BOOL_ONE", "1").unwrap();
                env.set_var("BOOL_ONE_QUOTED", "\'1\'").unwrap();
                env.set_var("BOOL_ONE_DOUBLE_QUOTED", "\"1\"").unwrap();

                for (key, value) in env.vars() {
                    assert!(
                        env.var_bool(&key).unwrap(),
                        "Expected {key:?}={value:?} to be true",
                    );
                }
            }

            pub(crate) fn var_bool_should_accept_falsy_values(env: &impl Env) {
                env.set_var("BOOL_FALSE", "false").unwrap();
                env.set_var("BOOL_FALSE_QUOTED", "\'false\'").unwrap();
                env.set_var("BOOL_FALSE_DOUBLE_QUOTED", "\"false\"")
                    .unwrap();
                env.set_var("BOOL_FALSE_UPPER", "FALSE").unwrap();
                env.set_var("BOOL_FALSE_MIXED", "FalsE").unwrap();
                env.set_var("BOOL_OFF", "off").unwrap();
                env.set_var("BOOL_OFF_QUOTED", "\'off\'").unwrap();
                env.set_var("BOOL_OFF_DOUBLE_QUOTED", "\"off\"").unwrap();
                env.set_var("BOOL_OFF_UPPER", "OFF").unwrap();
                env.set_var("BOOL_OFF_MIXED", "Off").unwrap();
                env.set_var("BOOL_ZERO", "0").unwrap();
                env.set_var("BOOL_ZERO_QUOTED", "\'0\'").unwrap();
                env.set_var("BOOL_ZERO_DOUBLE_QUOTED", "\"0\"").unwrap();
                env.set_var("BOOL_EMPTY", "").unwrap();
                env.set_var("BOOL_EMPTY_QUOTED", "\'\'").unwrap();
                env.set_var("BOOL_EMPTY_DOUBLE_QUOTED", "\"\"").unwrap();

                for (key, value) in env.vars() {
                    assert!(
                        !env.var_bool(&key).unwrap(),
                        "Expected {key:?}={value:?} to be false",
                    );
                }
            }

            pub(crate) fn var_bool_should_trim_value_before_unquoting(env: &impl Env) {
                env.set_var("BOOL_SPACES", " true ").unwrap();
                env.set_var("BOOL_TAB_NEWLINE", "\ton\n").unwrap();
                env.set_var("BOOL_SPACES_QUOTED", " '1' ").unwrap();
                env.set_var("BOOL_SPACES_DOUBLE_QUOTED", "\t\"true\"\r\n")
                    .unwrap();

                for (key, value) in env.vars() {
                    assert!(
                        env.var_bool(&key).unwrap(),
                        "Expected {key:?}={value:?} to be true",
                    );
                }
            }

            pub(crate) fn var_bool_should_reject_invalid_values(env: &impl Env) {
                env.set_var("BOOL_INVALID_STRING", "invalid").unwrap();
                env.set_var("BOOL_INVALID_NUMBER", "2").unwrap();
                env.set_var("BOOL_INVALID_DOUBLY_QUOTED_SINGLE", "''true''")
                    .unwrap();
                env.set_var("BOOL_INVALID_DOUBLY_QUOTED_MIXED", "\"'false'\"")
                    .unwrap();
                env.set_var("BOOL_INVALID_SPACES_IN_QUOTES", "' true '")
                    .unwrap();
                env.set_var("BOOL_INVALID_UTF8", invalid_utf8()).unwrap();

                for (key, value) in env.vars() {
                    assert!(
                        matches!(env.var_bool(&key), Err(e) if e.to_string().starts_with("[Env] Invalid boolean value for key")),
                        "Expected {key:?}={value:?} to be invalid"
                    );
                }
            }

            pub(crate) fn var_bool_should_return_false_for_missing_key(env: &impl Env) {
                // WHY: the key is &OsString to prevent the generic var_bool being instantiated
                // for &str and seemingly missing coverage due to monomorphization.
                let missing = &OsString::from("NON_EXISTENT_ENV_VAR");
                assert!(
                    !env.var_bool(missing).unwrap(),
                    "Expected var_bool() on non-existent var to be false"
                );
            }
        }

        pub(crate) mod vars {
            use super::*;

            pub(crate) fn vars_should_include_set_variable(env: &impl Env) {
                env.set_var("KEY", "value").unwrap();

                assert_eq!(
                    env.vars().collect::<Vec<_>>(),
                    [("KEY".into(), "value".into())]
                );
            }

            pub(crate) fn vars_should_exclude_removed_variable(env: &impl Env) {
                env.set_var("KEY", "value").unwrap();
                env.set_var("KEY2", "value").unwrap();

                env.remove_var("KEY");

                assert_eq!(
                    env.vars().collect::<Vec<_>>(),
                    [("KEY2".into(), "value".into())]
                );
            }

            pub(crate) fn vars_should_return_snapshot(env: &impl Env) {
                env.set_var("KEY", "value").unwrap();

                let vars = env.vars();
                env.set_var("KEY", "changed").unwrap();
                env.set_var("KEY2", "value").unwrap();

                assert_eq!(vars.collect::<Vec<_>>(), [("KEY".into(), "value".into())]);
            }
        }
    }

    mod env_error {
        use crate::ErrorKind;

        #[test]
        fn env_error_should_format_arguments_into_env_error() {
            let error = env_error!("{} {}", "some", "error");
            assert_eq!(error.kind(), ErrorKind::Env);
            assert_eq!(error.description(), "some error");
            assert_eq!(format!("{error}"), "[Env] some error");
        }
    }

    mod is_dir {
        use super::*;
        use crate::TestDir;

        #[test]
        fn is_dir_should_accept_existing_directory() {
            let dir = TestDir::new();

            assert!(is_dir(dir.path()).is_ok());
        }

        #[test]
        fn is_dir_should_reject_file_as_not_a_directory() {
            let dir = TestDir::new();
            let file = dir.path().join("file");
            std::fs::write(&file, "").unwrap();

            assert_eq!(
                is_dir(&file).unwrap_err().kind(),
                std::io::ErrorKind::NotADirectory
            );
        }

        #[test]
        fn is_dir_missing_path_should_return_not_found() {
            let dir = TestDir::new();

            assert_eq!(
                is_dir(&dir.path().join("missing")).unwrap_err().kind(),
                std::io::ErrorKind::NotFound
            );
        }
    }

    mod first_of_each_key {
        use super::*;

        #[test]
        fn first_of_each_key_should_keep_first_of_duplicate_keys() {
            let vars = vec![
                ("A".into(), "1".into()),
                ("B".into(), "2".into()),
                ("A".into(), "3".into()),
            ];

            assert_eq!(
                first_of_each_key(vars),
                [("A".into(), "1".into()), ("B".into(), "2".into())]
            );
        }

        #[cfg(windows)]
        #[test]
        fn first_of_each_key_should_match_keys_ignoring_ascii_case() {
            let vars = vec![("Path".into(), "1".into()), ("PATH".into(), "2".into())];

            assert_eq!(first_of_each_key(vars), [("Path".into(), "1".into())]);
        }
    }

    mod last_of_each_key {
        use super::*;

        #[test]
        fn last_of_each_key_should_keep_last_of_duplicate_keys() {
            let vars = vec![
                ("A".into(), "1".into()),
                ("B".into(), "2".into()),
                ("A".into(), "3".into()),
            ];

            assert_eq!(
                last_of_each_key(vars),
                [("B".into(), "2".into()), ("A".into(), "3".into())]
            );
        }
    }

    mod env_args {
        use super::*;

        fn args() -> Vec<OsString> {
            vec!["a".into(), "b".into()]
        }

        mod from {
            use super::*;

            #[test]
            fn from_should_iterate_arguments_of_slice() {
                let args = args();

                assert_eq!(
                    EnvArgs::from(args.as_slice()).collect::<Vec<_>>(),
                    ["a", "b"]
                );
            }
        }

        mod iterator {
            use super::*;

            mod next {
                use super::*;

                #[test]
                fn next_should_return_arguments_in_order() {
                    let args = args();
                    let mut iter = EnvArgs::from(args.as_slice());

                    assert_eq!(iter.next(), Some(&"a".into()));
                    assert_eq!(iter.next(), Some(&"b".into()));
                    assert_eq!(iter.next(), None);
                }
            }

            mod size_hint {
                use super::*;

                #[test]
                fn size_hint_should_return_number_of_remaining_arguments() {
                    let args = args();
                    let mut iter = EnvArgs::from(args.as_slice());
                    iter.next();

                    assert_eq!(iter.size_hint(), (1, Some(1)));
                }
            }
        }

        mod double_ended_iterator {
            use super::*;

            mod next_back {
                use super::*;

                #[test]
                fn next_back_should_return_arguments_in_reverse_order() {
                    let args = args();
                    let mut iter = EnvArgs::from(args.as_slice());

                    assert_eq!(iter.next_back(), Some(&"b".into()));
                    assert_eq!(iter.next_back(), Some(&"a".into()));
                    assert_eq!(iter.next_back(), None);
                }
            }
        }
    }

    mod env_vars {
        use super::*;

        fn vars() -> Vec<(OsString, OsString)> {
            vec![("A".into(), "1".into()), ("B".into(), "2".into())]
        }

        mod from {
            use super::*;

            #[test]
            fn from_should_iterate_variables_of_vec() {
                assert_eq!(EnvVars::from(vars()).collect::<Vec<_>>(), vars());
            }
        }

        mod from_iterator {
            use super::*;

            mod from_iter {
                use super::*;

                #[test]
                fn from_iter_should_iterate_collected_variables() {
                    let iter: EnvVars = vars().into_iter().collect();

                    assert_eq!(iter.collect::<Vec<_>>(), vars());
                }
            }
        }

        mod iterator {
            use super::*;

            mod next {
                use super::*;

                #[test]
                fn next_should_return_variables_in_order() {
                    let mut iter = EnvVars::from(vars());

                    assert_eq!(iter.next(), Some(("A".into(), "1".into())));
                    assert_eq!(iter.next(), Some(("B".into(), "2".into())));
                    assert_eq!(iter.next(), None);
                }
            }

            mod size_hint {
                use super::*;

                #[test]
                fn size_hint_should_return_number_of_remaining_variables() {
                    let mut iter = EnvVars::from(vars());
                    iter.next();

                    assert_eq!(iter.size_hint(), (1, Some(1)));
                }
            }
        }

        mod double_ended_iterator {
            use super::*;

            mod next_back {
                use super::*;

                #[test]
                fn next_back_should_return_variables_in_reverse_order() {
                    let mut iter = EnvVars::from(vars());

                    assert_eq!(iter.next_back(), Some(("B".into(), "2".into())));
                    assert_eq!(iter.next_back(), Some(("A".into(), "1".into())));
                    assert_eq!(iter.next_back(), None);
                }
            }
        }
    }

    mod env_impl {
        use super::*;

        #[test]
        fn env_impl_clone_should_share_changes() {
            let env = EnvImpl::new(Vec::new(), PathBuf::from("/a"));
            let clone = env.clone();

            env.set_var("A", "1").unwrap();
            env.replace_current_dir(PathBuf::from("/b"));

            assert_eq!(clone.var("A"), Some("1".into()));
            assert_eq!(clone.current_dir(), Path::new("/b"));
        }

        mod new_from_std {
            use super::*;

            #[test]
            fn new_from_std_should_load_nothing_before_use() {
                let env = EnvImpl::new_from_std();

                assert!(env.0.args.get().is_none());
                assert!(env.0.current_dir.get().is_none());
                assert!(env.0.current_exe.get().is_none());
                assert!(env.0.vars.get().is_none());
            }

            #[test]
            fn new_from_std_should_load_each_part_of_process_on_first_use() {
                let env = EnvImpl::new_from_std();

                assert_eq!(
                    env.vars().collect::<Vec<_>>(),
                    first_of_each_key(std::env::vars_os().collect())
                );
                assert!(env.0.current_dir.get().is_none());
                assert_eq!(
                    env.current_dir_arc().as_ref(),
                    std::env::current_dir().unwrap_or_default()
                );
                assert!(env.0.args.get().is_none());
                assert_eq!(
                    env.args().cloned().collect::<Vec<_>>(),
                    std::env::args_os().collect::<Vec<_>>()
                );
                assert!(env.0.current_exe.get().is_none());
                assert_eq!(
                    env.current_exe(),
                    std::env::current_exe().unwrap_or_default()
                );
            }
        }

        mod replace_args {
            use super::*;

            #[test]
            fn replace_args_should_replace_arguments() {
                let mut env = EnvImpl::new(Vec::new(), PathBuf::new());

                env.replace_args(vec!["app".into()]);

                assert_eq!(env.args().collect::<Vec<_>>(), ["app"]);
            }

            #[test]
            #[should_panic(expected = "an env is not shared while it is being built")]
            fn replace_args_shared_env_should_panic() {
                let mut env = EnvImpl::new(Vec::new(), PathBuf::new());
                let _clone = env.clone();

                env.replace_args(Vec::new());
            }
        }

        mod replace_current_dir {
            use super::*;

            #[test]
            fn replace_current_dir_should_replace_current_directory() {
                let env = EnvImpl::new(Vec::new(), PathBuf::from("/a"));

                env.replace_current_dir(PathBuf::from("/b"));

                assert_eq!(env.current_dir(), Path::new("/b"));
            }

            #[test]
            #[should_panic(expected = "the current directory should have a root")]
            fn replace_current_dir_relative_should_panic() {
                let env = EnvImpl::new(Vec::new(), PathBuf::from("/a"));

                env.replace_current_dir(PathBuf::from("b"));
            }
        }

        mod replace_current_exe {
            use super::*;

            #[test]
            fn replace_current_exe_should_replace_current_executable() {
                let mut env = EnvImpl::new(Vec::new(), PathBuf::new());

                env.replace_current_exe(PathBuf::from("/app"));

                assert_eq!(env.current_exe(), Path::new("/app"));
            }
        }

        mod replace_vars {
            use super::*;

            #[test]
            fn replace_vars_should_replace_all_variables() {
                let env = EnvImpl::new(vec![("A".into(), "1".into())], PathBuf::new());

                env.replace_vars(vec![("B".into(), "2".into())]);

                assert_eq!(env.vars().collect::<Vec<_>>(), [("B".into(), "2".into())]);
                assert_eq!(read(env.vars_lock()).generation, 2);
            }

            #[test]
            fn replace_vars_duplicate_keys_should_keep_last() {
                let env = EnvImpl::new(Vec::new(), PathBuf::new());

                env.replace_vars(vec![("A".into(), "1".into()), ("A".into(), "2".into())]);

                assert_eq!(env.var("A"), Some("2".into()));
                assert_eq!(env.remove_var("A"), Some("2".into()));
                assert_eq!(env.var("A"), None);
            }

            #[test]
            #[should_panic(expected = "Invalid environment variable key 'A=B'")]
            fn replace_vars_invalid_var_should_panic() {
                let env = EnvImpl::new(Vec::new(), PathBuf::new());

                env.replace_vars(vec![("A=B".into(), "1".into())]);
            }
        }

        mod set_current_dir_checked {
            use super::*;

            fn accept(_: &Path) -> std::io::Result<()> {
                Ok(())
            }

            #[test]
            fn set_current_dir_checked_should_check_resolved_path() {
                let env = EnvImpl::new(Vec::new(), PathBuf::from("/a"));

                env.set_current_dir_checked(Path::new("b/../c"), &|path| {
                    (path == Path::new("/a/c"))
                        .then_some(())
                        .ok_or(std::io::ErrorKind::NotFound.into())
                })
                .unwrap();

                assert_eq!(env.current_dir(), Path::new("/a/c"));
            }

            #[test]
            fn set_current_dir_checked_failing_check_should_keep_current_dir() {
                let env = EnvImpl::new(Vec::new(), PathBuf::from("/a"));

                let error = env
                    .set_current_dir_checked(Path::new("b"), &|_| {
                        Err(std::io::ErrorKind::NotFound.into())
                    })
                    .unwrap_err();

                #[cfg(unix)]
                let error_text = "Current directory could not be set, entity not found: /a/b";
                #[cfg(windows)]
                let error_text = "Current directory could not be set, entity not found: \\a\\b";

                assert_eq!(error.description(), error_text);
                assert_eq!(
                    error.source_as::<std::io::Error>().unwrap().kind(),
                    std::io::ErrorKind::NotFound
                );
                assert_eq!(env.current_dir(), Path::new("/a"));
            }

            #[test]
            fn set_current_dir_checked_relative_should_resolve_against_previous_current_dir() {
                let base = std::env::temp_dir().join("a");

                for (path, expected) in [
                    ("b", base.join("b")),
                    ("./b/.", base.join("b")),
                    ("b/../c", base.join("c")),
                    ("../b", std::env::temp_dir().join("b")),
                ] {
                    let env = EnvImpl::new(Vec::new(), base.clone());

                    env.set_current_dir_checked(Path::new(path), &accept)
                        .unwrap();

                    assert_eq!(env.current_dir(), expected, "path: {path}");
                }
            }

            #[test]
            fn set_current_dir_checked_should_not_go_above_root() {
                let current_dir = std::env::temp_dir();
                let env = EnvImpl::new(Vec::new(), current_dir.clone());
                let root = current_dir.ancestors().last().unwrap().to_path_buf();
                let up = vec![".."; current_dir.components().count() + 1].join("/");

                env.set_current_dir_checked(Path::new(&up), &accept)
                    .unwrap();

                assert_eq!(env.current_dir(), root);
            }

            #[test]
            fn set_current_dir_checked_relative_unknown_current_dir_should_fail() {
                let env = EnvImpl::new(Vec::new(), PathBuf::new());

                let error = env
                    .set_current_dir_checked(Path::new("a"), &accept)
                    .unwrap_err();

                assert_eq!(
                    error.description(),
                    "Current directory could not be set, the current directory is not known: a"
                );
                assert_eq!(env.current_dir(), Path::new(""));
            }

            #[test]
            fn set_current_dir_checked_absolute_unknown_current_dir_should_set_it() {
                let env = EnvImpl::new(Vec::new(), PathBuf::new());

                env.set_current_dir_checked(&std::env::temp_dir(), &accept)
                    .unwrap();

                assert_eq!(env.current_dir(), std::env::temp_dir());
            }

            #[test]
            fn set_current_dir_checked_current_dir_changed_during_check_should_check_again() {
                let env = EnvImpl::new(Vec::new(), PathBuf::from("/a"));
                let checked = std::sync::Mutex::new(Vec::new());
                let change = std::sync::Once::new();

                env.set_current_dir_checked(Path::new("c"), &|path| {
                    checked.lock().unwrap().push(path.to_path_buf());
                    change.call_once(|| env.replace_current_dir(PathBuf::from("/b")));
                    Ok(())
                })
                .unwrap();

                assert_eq!(
                    checked.into_inner().unwrap(),
                    [PathBuf::from("/a/c"), PathBuf::from("/b/c")]
                );
                assert_eq!(env.current_dir(), Path::new("/b/c"));
            }
        }

        #[cfg(feature = "process")]
        mod changed_vars {
            use super::*;

            #[test]
            fn changed_vars_should_return_none_while_vars_of_process() {
                let env = EnvImpl::new_from_std();
                assert_eq!(env.changed_vars(), None);

                env.var("PATH");

                assert_eq!(env.changed_vars(), None);
            }

            #[test]
            fn changed_vars_should_return_vars_after_change() {
                let env = EnvImpl::new_from_std();

                env.remove_var("PATH");

                assert_eq!(env.changed_vars(), Some(env.vars().collect()));
            }

            #[test]
            fn changed_vars_all_removed_should_return_no_vars() {
                let env = EnvImpl::new_from_std();

                for (key, _) in env.vars() {
                    env.remove_var(key);
                }

                assert_eq!(env.changed_vars(), Some(Vec::new()));
            }

            #[test]
            fn changed_vars_var_set_to_same_value_should_return_none() {
                let env = EnvImpl::new_from_std();
                // WHY: Windows has hidden variables like `=C:` that cannot be set.
                let (key, value) = env
                    .vars()
                    .find(|(key, _)| !key.to_string_lossy().contains('='))
                    .expect("the process has variables");

                env.set_var(key, value).unwrap();

                assert_eq!(env.changed_vars(), None);
            }

            #[test]
            fn changed_vars_should_return_given_vars() {
                let env = EnvImpl::new(vec![("A".into(), "1".into())], PathBuf::new());

                assert_eq!(env.changed_vars(), Some(vec![("A".into(), "1".into())]));
            }
        }

        mod env {
            use super::*;
            use crate::TestDir;

            fn empty() -> EnvImpl {
                EnvImpl::new(Vec::new(), PathBuf::new())
            }

            mod args {
                use super::*;

                #[test]
                fn args_should_return_arguments_in_order() {
                    let mut env = empty();
                    env.replace_args(vec!["app".into(), "--flag".into()]);

                    crate::env::tests::env::args::args_should_return_arguments_in_order(
                        &env,
                        &["app", "--flag"],
                    );
                }

                #[test]
                fn args_should_return_process_arguments() {
                    let env = EnvImpl::new_from_std();
                    assert!(env.0.args.get().is_none());
                    assert_eq!(
                        env.args().cloned().collect::<Vec<_>>(),
                        std::env::args_os().collect::<Vec<_>>()
                    );
                }
            }

            mod current_dir {
                use super::*;

                #[test]
                fn current_dir_should_return_current_directory() {
                    let env = EnvImpl::new(Vec::new(), PathBuf::from("/a"));

                    crate::env::tests::env::current_dir::current_dir_should_return_current_directory(
                        &env,
                        Path::new("/a"),
                    );
                }

                #[test]
                fn current_dir_should_return_process_current_directory() {
                    let env = EnvImpl::new_from_std();
                    let expected = std::env::current_dir().unwrap_or_default();
                    assert_eq!(env.current_dir(), expected.as_path());
                }
            }

            mod current_exe {
                use super::*;

                #[test]
                fn current_exe_should_return_current_executable() {
                    let mut env = empty();
                    env.replace_current_exe(PathBuf::from("/app"));

                    crate::env::tests::env::current_exe::current_exe_should_return_current_executable(
                        &env,
                        Path::new("/app"),
                    );
                }

                #[test]
                fn current_exe_should_return_process_current_executable() {
                    let env = EnvImpl::new_from_std();
                    assert!(env.0.current_exe.get().is_none());
                    let expected = std::env::current_exe().unwrap_or_default();
                    assert_eq!(env.current_exe(), expected.as_path());
                }
            }

            mod home_dir {
                use super::*;

                #[test]
                fn home_dir_should_return_process_home_directory() {
                    let env = EnvImpl::new_from_std();
                    let expected = std::env::var_os(HOME_VAR)
                        .filter(|home| !home.is_empty())
                        .map(PathBuf::from);
                    assert_eq!(env.home_dir(), expected);
                }

                #[test]
                fn home_dir_should_return_home_variable() {
                    crate::env::tests::env::home_dir::home_dir_should_return_home_variable(&empty());
                }

                #[test]
                fn home_dir_unset_or_empty_variable_should_return_none() {
                    crate::env::tests::env::home_dir::home_dir_unset_or_empty_variable_should_return_none(&empty());
                }
            }

            mod remove_var {
                use super::*;

                #[test]
                fn remove_var_should_return_value_for_present_key() {
                    crate::env::tests::env::remove_var::remove_var_should_return_value_for_present_key(&empty());
                }

                #[test]
                fn remove_var_should_increment_generation() {
                    let env = EnvImpl::new(vec![("A".into(), "1".into())], PathBuf::new());

                    env.remove_var("A");

                    assert_eq!(read(env.vars_lock()).generation, 2);
                }

                #[test]
                fn remove_var_should_return_none_for_missing_key() {
                    crate::env::tests::env::remove_var::remove_var_should_return_none_for_missing_key(&empty());
                }

                #[test]
                fn remove_var_missing_key_should_keep_generation() {
                    let env = EnvImpl::new_from_std();

                    env.remove_var("AGPLATFORM_MISSING_VAR");

                    assert_eq!(read(env.vars_lock()).generation, 0);
                }

                #[cfg(windows)]
                #[test]
                fn remove_var_should_match_key_ignoring_ascii_case() {
                    crate::env::tests::env::remove_var::remove_var_should_match_key_ignoring_ascii_case(&empty());
                }

                #[cfg(unix)]
                #[test]
                fn remove_var_should_match_key_case_sensitively() {
                    crate::env::tests::env::remove_var::remove_var_should_match_key_case_sensitively(&empty());
                }
            }

            mod set_current_dir {
                use super::*;

                #[test]
                fn set_current_dir_should_return_previous_value() {
                    let dir = TestDir::new();

                    crate::env::tests::env::set_current_dir::set_current_dir_should_return_previous_value(
                        &EnvImpl::new_from_std(),
                        dir.path(),
                    );
                }

                #[test]
                fn set_current_dir_should_update_current_directory() {
                    let dir = TestDir::new();

                    crate::env::tests::env::set_current_dir::set_current_dir_should_update_current_directory(
                        &EnvImpl::new_from_std(),
                        dir.path(),
                    );
                }

                #[test]
                fn set_current_dir_relative_should_resolve_against_previous_current_dir() {
                    let dir = TestDir::new();
                    std::fs::create_dir(dir.path().join("a")).unwrap();

                    crate::env::tests::env::set_current_dir::set_current_dir_relative_should_resolve_against_previous_current_dir(
                        &EnvImpl::new_from_std(),
                        dir.path(),
                    );
                }

                #[test]
                fn set_current_dir_should_not_go_above_root() {
                    let dir = TestDir::new();

                    crate::env::tests::env::set_current_dir::set_current_dir_should_not_go_above_root(
                        &EnvImpl::new_from_std(),
                        dir.path(),
                    );
                }

                #[test]
                fn set_current_dir_missing_directory_should_fail_and_keep_current_dir() {
                    let env = EnvImpl::new_from_std();
                    let dir = TestDir::new();
                    env.set_current_dir(dir.path()).unwrap();

                    let error = env.set_current_dir("missing").unwrap_err();

                    assert_eq!(
                        error.source_as::<std::io::Error>().unwrap().kind(),
                        std::io::ErrorKind::NotFound
                    );
                    assert_eq!(env.current_dir(), dir.path());
                }

                #[test]
                fn set_current_dir_file_should_fail_and_keep_current_dir() {
                    let env = EnvImpl::new_from_std();
                    let dir = TestDir::new();
                    std::fs::write(dir.path().join("file"), "").unwrap();
                    env.set_current_dir(dir.path()).unwrap();

                    let error = env.set_current_dir(dir.path().join("file")).unwrap_err();

                    assert_eq!(
                        error.source_as::<std::io::Error>().unwrap().kind(),
                        std::io::ErrorKind::NotADirectory
                    );
                    assert_eq!(env.current_dir(), dir.path());
                }

                #[test]
                fn set_current_dir_symlink_to_directory_should_follow_it() {
                    let env = EnvImpl::new_from_std();
                    let dir = TestDir::new();
                    let link = dir.path().join("link");
                    std::fs::create_dir(dir.path().join("target")).unwrap();
                    #[cfg(unix)]
                    std::os::unix::fs::symlink("target", &link).unwrap();
                    #[cfg(windows)]
                    std::os::windows::fs::symlink_dir("target", &link).unwrap();

                    env.set_current_dir(&link).unwrap();

                    assert_eq!(env.current_dir(), link);
                }
            }

            mod set_var {
                use super::*;

                #[test]
                fn set_var_should_increment_generation() {
                    let env = empty();

                    env.set_var("A", "1").unwrap();

                    assert_eq!(read(env.vars_lock()).generation, 2);
                }

                #[test]
                fn set_var_same_value_should_keep_generation() {
                    let env = EnvImpl::new(vec![("A".into(), "1".into())], PathBuf::new());

                    let previous = env.set_var("A", "1").unwrap();

                    assert_eq!(previous, Some("1".into()));
                    assert_eq!(read(env.vars_lock()).generation, 1);
                }

                #[test]
                fn set_var_invalid_key_should_fail_and_keep_vars() {
                    crate::env::tests::env::set_var::set_var_invalid_key_should_fail_and_keep_vars(
                        &empty(),
                    );
                }

                #[test]
                fn set_var_invalid_var_should_keep_generation() {
                    let env = empty();

                    env.set_var("A=B", "value").unwrap_err();
                    env.set_var("A", "a\0b").unwrap_err();

                    assert_eq!(read(env.vars_lock()).generation, 1);
                }

                #[test]
                fn set_var_value_with_nul_should_fail_and_keep_vars() {
                    crate::env::tests::env::set_var::set_var_value_with_nul_should_fail_and_keep_vars(&empty());
                }

                #[test]
                fn set_var_should_return_none_for_new_key() {
                    crate::env::tests::env::set_var::set_var_should_return_none_for_new_key(
                        &empty(),
                    );
                }

                #[test]
                fn set_var_should_return_previous_value_for_existing_key() {
                    crate::env::tests::env::set_var::set_var_should_return_previous_value_for_existing_key(&empty());
                }

                #[cfg(windows)]
                #[test]
                fn set_var_different_case_should_replace_value_keeping_key() {
                    crate::env::tests::env::set_var::set_var_different_case_should_replace_value_keeping_key(&empty());
                }

                #[cfg(unix)]
                #[test]
                fn set_var_different_case_should_add_variable() {
                    crate::env::tests::env::set_var::set_var_different_case_should_add_variable(
                        &empty(),
                    );
                }
            }

            mod tmp_dir {
                use super::*;

                #[test]
                fn tmp_dir_should_return_process_temp_directory() {
                    let env = EnvImpl::new_from_std();
                    assert_eq!(env.tmp_dir(), std::env::temp_dir());
                }

                #[cfg(unix)]
                #[test]
                fn tmp_dir_should_return_tmpdir_variable() {
                    crate::env::tests::env::tmp_dir::tmp_dir_should_return_tmpdir_variable(&empty());
                }

                #[cfg(unix)]
                #[test]
                fn tmp_dir_unset_variable_should_return_default() {
                    crate::env::tests::env::tmp_dir::tmp_dir_unset_variable_should_return_default(
                        &empty(),
                    );
                }

                #[cfg(windows)]
                #[test]
                fn tmp_dir_should_return_first_set_variable_with_separator() {
                    crate::env::tests::env::tmp_dir::tmp_dir_should_return_first_set_variable_with_separator(&empty());
                }
            }

            mod var {
                use super::*;

                #[test]
                fn var_should_return_none_for_missing_key() {
                    crate::env::tests::env::var::var_should_return_none_for_missing_key(&empty());
                }

                #[test]
                fn var_should_return_value_for_present_key() {
                    crate::env::tests::env::var::var_should_return_value_for_present_key(&empty());
                }

                #[test]
                fn var_should_return_value_that_is_not_utf8() {
                    crate::env::tests::env::var::var_should_return_value_that_is_not_utf8(&empty());
                }

                #[cfg(windows)]
                #[test]
                fn var_should_match_key_ignoring_ascii_case() {
                    crate::env::tests::env::var::var_should_match_key_ignoring_ascii_case(&empty());
                }

                #[cfg(unix)]
                #[test]
                fn var_should_match_key_case_sensitively() {
                    crate::env::tests::env::var::var_should_match_key_case_sensitively(&empty());
                }
            }

            mod var_bool {
                use super::*;

                #[test]
                fn var_bool_should_accept_truthy_values() {
                    crate::env::tests::env::var_bool::var_bool_should_accept_truthy_values(&empty());
                }

                #[test]
                fn var_bool_should_accept_falsy_values() {
                    crate::env::tests::env::var_bool::var_bool_should_accept_falsy_values(&empty());
                }

                #[test]
                fn var_bool_should_trim_value_before_unquoting() {
                    crate::env::tests::env::var_bool::var_bool_should_trim_value_before_unquoting(
                        &empty(),
                    );
                }

                #[test]
                fn var_bool_should_reject_invalid_values() {
                    crate::env::tests::env::var_bool::var_bool_should_reject_invalid_values(
                        &empty(),
                    );
                }

                #[test]
                fn var_bool_should_return_false_for_missing_key() {
                    crate::env::tests::env::var_bool::var_bool_should_return_false_for_missing_key(
                        &empty(),
                    );
                }
            }

            mod vars {
                use super::*;

                #[test]
                fn vars_should_include_set_variable() {
                    crate::env::tests::env::vars::vars_should_include_set_variable(&empty());
                }

                #[test]
                fn vars_should_exclude_removed_variable() {
                    crate::env::tests::env::vars::vars_should_exclude_removed_variable(&empty());
                }

                #[test]
                fn vars_should_return_snapshot() {
                    crate::env::tests::env::vars::vars_should_return_snapshot(&empty());
                }
            }
        }
    }

    /// Returns a value that is not valid UTF-8.
    fn invalid_utf8() -> OsString {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            OsString::from_vec(vec![b'a', 0xff])
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStringExt;
            OsString::from_wide(&[u16::from(b'a'), 0xd800])
        }
    }
}
