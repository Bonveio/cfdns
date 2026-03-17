// Build script for cfdns.
// Provides C stub implementations for missing Android API 19 symbols.

fn main() {
    // Android API 19 is missing several libc symbols that Rust's std and
    // our dependencies expect. We provide no-op / error-returning stubs so
    // the linker succeeds. At runtime these code paths are either unreachable
    // (panic=abort eliminates backtrace) or degrade gracefully.
    let target = std::env::var("TARGET").unwrap_or_default();
    let api = std::env::var("ANDROID_API_LEVEL").unwrap_or_default();

    if target.contains("android") && api == "19" {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let stubs_path = format!("{out_dir}/android_api19_stubs.c");

        std::fs::write(
            &stubs_path,
            r#"
// Stub implementations for symbols missing in Android API 19 bionic.
// These are either dead code (panic=abort removes backtrace usage) or
// degrade gracefully (rpassword handles errors, signal returns SIG_ERR).

#include <stddef.h>
#include <errno.h>

// dl_iterate_phdr: used by Rust's backtrace. With panic=abort this is
// dead code but the linker still needs the symbol.
int dl_iterate_phdr(int (*callback)(void *info, size_t size, void *data),
                    void *data) {
    (void)callback;
    (void)data;
    return 0;
}

// signal: used by Rust's std for signal handling setup.
// Return SIG_ERR (-1 cast to function pointer) to indicate failure.
typedef void (*sighandler_t)(int);
sighandler_t signal(int signum, sighandler_t handler) {
    (void)signum;
    (void)handler;
    errno = 22; // EINVAL
    return (sighandler_t)-1; // SIG_ERR
}

// tcgetattr / tcsetattr: used by rpassword for terminal raw mode.
// Return -1 with ENOSYS to indicate "not supported".
struct termios;
int tcgetattr(int fd, struct termios *termios_p) {
    (void)fd;
    (void)termios_p;
    errno = 38; // ENOSYS
    return -1;
}

int tcsetattr(int fd, int optional_actions, const struct termios *termios_p) {
    (void)fd;
    (void)optional_actions;
    (void)termios_p;
    errno = 38; // ENOSYS
    return -1;
}
"#,
        )
        .expect("Failed to write stubs file");

        cc::Build::new().file(&stubs_path).compile("android_stubs");
    }
}
