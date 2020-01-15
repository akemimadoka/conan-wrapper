use crate::ConanBuildInfo;
use std::collections::HashMap;
use std::env;

lazy_static! {
    static ref CARGO_OS_TO_CONAN_OS: HashMap<&'static str, &'static str> = hashmap!(
        "windows" => "Windows",
        "linux" => "Linux",
        "macos" => "Macos",
        "android" => "Android",
        "ios" => "iOS",
        "freebsd" => "FreeBSD"
    );
    static ref CARGO_ARCH_TO_CONAN_ARCH: HashMap<&'static str, &'static str> = hashmap!(
        "powerpc" => "ppc32",
        "powerpc64" => "ppc64",
        "arm" => "armv7",
        "aarch64" => "armv8"
    );
}

#[cfg(feature = "cargo")]
pub fn cargo_os_to_conan_os(os_name: &str) -> &str {
    CARGO_OS_TO_CONAN_OS.get(os_name).unwrap_or(&os_name)
}

// TODO: Some arch contains endian information
#[cfg(feature = "cargo")]
pub fn cargo_arch_to_conan_arch(arch_name: &str) -> &str {
    CARGO_ARCH_TO_CONAN_ARCH
        .get(arch_name)
        .unwrap_or(&arch_name)
}

pub fn cargo_profile_to_conan_build_type(profile: &str) -> &str {
    match profile {
        "debug" => "Debug",
        "release" => "Release",
        _ => profile,
    }
}

#[cfg(feature = "cargo")]
pub fn auto_detect_settings_from_cargo() -> HashMap<String, String> {
    let mut result = HashMap::new();

    if let Ok(os) = env::var("CARGO_CFG_TARGET_OS") {
        result.insert("os".into(), cargo_os_to_conan_os(&os).into());
    }

    if let Ok(arch) = env::var("CARGO_CFG_TARGET_ARCH") {
        result.insert("arch".into(), cargo_arch_to_conan_arch(&arch).into());
    }

    if let Ok(profile) = env::var("PROFILE") {
        result.insert(
            "build_type".into(),
            cargo_profile_to_conan_build_type(profile.as_str()).into(),
        );
    }

    result
}

#[cfg(feature = "cargo")]
pub fn output_information_to_cargo(build_info: &ConanBuildInfo) {
    for dependency in &build_info.dependencies {
        for lib_path in &dependency.lib_paths {
            println!("cargo:rustc-link-search=native={}", lib_path);
        }
        for lib in &dependency.libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
        for system_lib in &dependency.system_libs {
            println!("cargo:rustc-link-lib={}", system_lib);
        }
    }
}
