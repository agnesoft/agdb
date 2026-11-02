use agbench::Bench;
use agplatform::Env;
use agplatform::Platform;
use agplatform::TestDir;
use std::hint::black_box;
use std::path::PathBuf;
use std::process::ExitCode;

/// The variable set by the fixture.
const KEY: &str = "AGBENCH_ENV_KEY";

fn main() -> ExitCode {
    agbench::main(&[
        args,
        current_dir,
        current_exe,
        home_dir,
        tmp_dir,
        var,
        vars,
        set_var,
        remove_var,
        set_current_dir,
    ])
}

/// The platform created after setting [`KEY`], so that both `std` and
/// `agplatform` find it last when scanning the environment. Removes [`KEY`]
/// and restores the current directory when dropped.
struct Fixture<P> {
    platform: P,
    current_dir: PathBuf,
    dir: TestDir,
}

fn fixture() -> Fixture<impl Platform> {
    set_std_var();

    Fixture {
        platform: agplatform::platform(),
        current_dir: std::env::current_dir().expect("current dir should be accessible"),
        dir: TestDir::temp(),
    }
}

impl<P: Platform> Fixture<P> {
    fn env(&self) -> &P::Env {
        self.platform.env()
    }
}

impl<P> Drop for Fixture<P> {
    fn drop(&mut self) {
        remove_std_var();
        // WHY: Windows cannot remove the current directory.
        let _ = std::env::set_current_dir(&self.current_dir);
    }
}

fn set_std_var() {
    // SAFETY: the benchmarks run on a single thread.
    unsafe { std::env::set_var(KEY, "value") };
}

fn remove_std_var() {
    // SAFETY: the benchmarks run on a single thread.
    unsafe { std::env::remove_var(KEY) };
}

fn args(bench: &mut Bench) {
    bench
        .benchmark("env/args", fixture)
        .fail_factor(0.5)
        .target("std", |_| {
            std::env::args().map(|arg| arg.len()).sum::<usize>()
        })
        .target("agplatform", |fx| {
            fx.env().args().map(|arg| arg.len()).sum::<usize>()
        })
        .run();
}

fn current_dir(bench: &mut Bench) {
    bench
        .benchmark("env/current_dir", fixture)
        .fail_factor(0.5)
        .target("std", |_| std::env::current_dir().unwrap())
        .target("agplatform", |fx| {
            black_box(fx.env().current_dir());
        })
        .run();
}

fn current_exe(bench: &mut Bench) {
    bench
        .benchmark("env/current_exe", fixture)
        .fail_factor(0.5)
        .target("std", |_| std::env::current_exe().unwrap())
        .target("agplatform", |fx| {
            black_box(fx.env().current_exe());
        })
        .run();
}

fn home_dir(bench: &mut Bench) {
    bench
        .benchmark("env/home_dir", fixture)
        .fail_factor(0.5)
        .target("std", |_| std::env::home_dir().unwrap())
        .target("agplatform", |fx| {
            black_box(fx.env().home_dir());
        })
        .run();
}

fn tmp_dir(bench: &mut Bench) {
    bench
        .benchmark("env/tmp_dir", fixture)
        .fail_factor(0.5)
        .target("std", |_| std::env::temp_dir())
        .target("agplatform", |fx| {
            black_box(fx.env().tmp_dir());
        })
        .run();
}

fn var(bench: &mut Bench) {
    bench
        .benchmark("env/var", fixture)
        .fail_factor(1.0)
        .target("std", |_| std::env::var(KEY).unwrap())
        .target("agplatform", |fx| {
            black_box(fx.env().var(KEY).unwrap());
        })
        .run();

    bench
        .benchmark("env/var/missing", fixture)
        .fail_factor(1.0)
        .target("std", |_| std::env::var("AGBENCH_ENV_MISSING").is_err())
        .target("agplatform", |fx| {
            fx.env().var("AGBENCH_ENV_MISSING").is_none()
        })
        .run();
}

fn vars(bench: &mut Bench) {
    bench
        .benchmark("env/vars", fixture)
        .fail_factor(0.5)
        .target("std", |_| {
            std::env::vars()
                .map(|(key, value)| key.len() + value.len())
                .sum::<usize>()
        })
        .target("agplatform", |fx| {
            fx.env()
                .vars()
                .map(|(key, value)| key.len() + value.len())
                .sum::<usize>()
        })
        .run();
}

fn set_var(bench: &mut Bench) {
    bench
        .benchmark("env/set_var", fixture)
        .fail_factor(0.75)
        .target("std", |_| set_std_var())
        .target("agplatform", |fx| fx.platform.env().set_var(KEY, "value"))
        .run();
}

fn remove_var(bench: &mut Bench) {
    bench
        .benchmark("env/remove_var", fixture)
        .fail_factor(0.7)
        .iteration_setup(|fx| {
            set_std_var();
            fx.platform.env().set_var(KEY, "value").unwrap();
        })
        .target("std", |_, ()| remove_std_var())
        .target("agplatform", |fx, ()| fx.platform.env().remove_var(KEY))
        .run();
}

fn set_current_dir(bench: &mut Bench) {
    // WHY: alternating, as Windows skips setting the directory that is
    // already current.
    bench
        .benchmark("env/set_current_dir", || {
            let fx = fixture();
            let dirs = ["a", "b"].map(|name| {
                let dir = fx.dir.path().join(name);
                std::fs::create_dir(&dir).expect("test dir should be writable");
                dir
            });
            (fx, dirs, 0)
        })
        .fail_factor(0.6)
        .target("std", |(_, dirs, next)| {
            *next ^= 1;
            std::env::set_current_dir(&dirs[*next]).unwrap()
        })
        .target("agplatform", |(fx, dirs, next)| {
            *next ^= 1;
            fx.platform.env().set_current_dir(&dirs[*next])
        })
        .run();
}
