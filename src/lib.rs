extern crate regex;
extern crate which;

extern crate serde;
extern crate serde_json;

use serde::Deserialize;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate maplit;

use regex::Regex;
use std::collections::HashMap;
use std::string::String;

use std::process::Command;

pub fn find_system_conan() -> Option<std::path::PathBuf> {
    which::which("conan").ok()
}

lazy_static! {
    static ref CONAN_VERSION_REGEX: Regex = Regex::new("Conan version ([\\d\\.]+)").unwrap();
}

pub fn determine_conan_version(path: &std::path::Path) -> Option<String> {
    let output = Command::new(path)
        .arg("--version")
        .output()
        .expect(&format!(
            "Cannot execute conan from path \"{}\"",
            path.display()
        ));
    let output_string = String::from_utf8(output.stdout).ok()?;
    let found_version = CONAN_VERSION_REGEX.captures(&output_string)?.get(1)?;
    Some(found_version.as_str().into())
}

#[derive(Debug)]
pub enum InstallTarget {
    ConanFile {
        path: String,
        reference: Option<String>,
    },
    Package {
        reference: String,
    },
}

#[derive(Debug)]
pub struct Generator(&'static str);

impl Generator {
    pub const CMAKE: Generator = Generator("cmake");
    pub const CMAKE_MULTI: Generator = Generator("cmake_multi");
    pub const CMAKE_PATHS: Generator = Generator("cmake_paths");
    pub const CMAKE_FIND_PACKAGE: Generator = Generator("cmake_find_package");
    pub const CMAKE_FIND_PACKAGE_MULTI: Generator = Generator("cmake_find_package_multi");
    pub const VISUAL_STUDIO: Generator = Generator("visual_studio");
    pub const VISUAL_STUDIO_MULTI: Generator = Generator("visual_studio_multi");
    pub const VISUAL_STUDIO_LEGACY: Generator = Generator("visual_studio_legacy");
    pub const XCODE: Generator = Generator("xcode");
    pub const COMPILER_ARGS: Generator = Generator("compiler_args");
    pub const GCC: Generator = Generator("gcc");
    pub const BOOST_BUILD: Generator = Generator("boost-build");
    pub const B2: Generator = Generator("b2");
    pub const QBS: Generator = Generator("qbs");
    pub const QMAKE: Generator = Generator("qmake");
    pub const SCONS: Generator = Generator("scons");
    pub const PKG_CONFIG: Generator = Generator("pkg_config");
    pub const VIRTUALENV: Generator = Generator("virtualenv");
    pub const VIRTUALENV_PYTHON: Generator = Generator("virtualenv_python");
    pub const VIRTUALBUILDENV: Generator = Generator("virtualbuildenv");
    pub const VIRTUALRUNENV: Generator = Generator("virtualrunenv");
    pub const YOUCOMPLETEME: Generator = Generator("youcompleteme");
    pub const TXT: Generator = Generator("txt");
    pub const JSON: Generator = Generator("json");
    pub const PREMAKE: Generator = Generator("premake");
    pub const MAKE: Generator = Generator("make");
    pub const DEPLOY: Generator = Generator("deploy");

    pub fn custom(name: &'static str) -> Generator {
        Generator(name)
    }
}

#[derive(Debug)]
pub enum BuildConfiguration {
    All,
    Never,
    Missing,
    Cascade,
    Outdated,
    Package { pattern: String },
}

#[derive(Debug)]
pub struct InstallArguments {
    pub install_target: InstallTarget,
    pub generators: Vec<Generator>,
    pub install_folder: String,
    pub no_imports: bool,
    pub build_configurations: Vec<BuildConfiguration>,
    pub envs: HashMap<String, String>,
    pub options: HashMap<String, String>,
    pub profile: Option<String>,
    pub remote: Option<String>,
    pub settings: HashMap<String, String>,
    pub check_update: bool,
}

impl InstallArguments {
    pub fn to_commandline_arguements(&self) -> Vec<String> {
        let mut result = Vec::<String>::new();

        match &self.install_target {
            InstallTarget::ConanFile { path, reference } => {
                result.push(path.clone());
                if let Some(reference) = reference {
                    result.push(reference.clone());
                }
            }
            InstallTarget::Package { reference } => {
                result.push(reference.clone());
            }
        }

        for generator in &self.generators {
            result.push("-g".into());
            result.push(generator.0.into());
        }

        result.push("-if".into());
        result.push(self.install_folder.clone());

        if self.no_imports {
            result.push("--no-imports".into());
        }

        for build_configuration in &self.build_configurations {
            result.push("--build".into());
            match build_configuration {
                BuildConfiguration::All => {}
                BuildConfiguration::Never => {
                    result.push("never".into());
                }
                BuildConfiguration::Missing => {
                    result.push("missing".into());
                }
                BuildConfiguration::Cascade => {
                    result.push("cascade".into());
                }
                BuildConfiguration::Outdated => {
                    result.push("outdated".into());
                }
                BuildConfiguration::Package { pattern } => {
                    result.push(pattern.clone());
                }
            }
        }

        for (env_key, env_value) in &self.envs {
            result.push("-e".into());
            result.push(format!("{}={}", env_key, env_value));
        }

        for (option_key, option_value) in &self.options {
            result.push("-o".into());
            result.push(format!("{}={}", option_key, option_value));
        }

        if let Some(profile) = &self.profile {
            result.push("-pr".into());
            result.push(profile.clone());
        }

        if let Some(remote) = &self.remote {
            result.push("-r".into());
            result.push(remote.clone());
        }

        for (setting_key, setting_value) in &self.settings {
            result.push("-s".into());
            result.push(format!("{}={}", setting_key, setting_value));
        }

        if self.check_update {
            result.push("--update".into());
        }

        result
    }
}

pub struct InstallArgumentsBuilder {
    install_target: InstallTarget,
    generators: Vec<Generator>,
    install_folder: String,
    no_imports: bool,
    build_configurations: Vec<BuildConfiguration>,
    envs: HashMap<String, String>,
    options: HashMap<String, String>,
    profile: Option<String>,
    remote: Option<String>,
    settings: HashMap<String, String>,
    check_update: bool,
}

impl InstallArgumentsBuilder {
    pub fn new(install_target: InstallTarget, install_folder: String) -> InstallArgumentsBuilder {
        InstallArgumentsBuilder {
            install_target,
            generators: Vec::new(),
            install_folder,
            no_imports: false,
            build_configurations: vec![BuildConfiguration::All],
            envs: HashMap::new(),
            options: HashMap::new(),
            profile: None,
            remote: None,
            settings: HashMap::new(),
            check_update: false,
        }
    }

    pub fn generators(&mut self, value: Vec<Generator>) -> &mut InstallArgumentsBuilder {
        self.generators = value;
        self
    }

    pub fn no_imports(&mut self, value: bool) -> &mut InstallArgumentsBuilder {
        self.no_imports = value;
        self
    }

    pub fn build_configurations(
        &mut self,
        value: Vec<BuildConfiguration>,
    ) -> &mut InstallArgumentsBuilder {
        self.build_configurations = value;
        self
    }

    pub fn envs(&mut self, value: HashMap<String, String>) -> &mut InstallArgumentsBuilder {
        self.envs = value;
        self
    }

    pub fn options(&mut self, value: HashMap<String, String>) -> &mut InstallArgumentsBuilder {
        self.options = value;
        self
    }

    pub fn profile(&mut self, value: String) -> &mut InstallArgumentsBuilder {
        self.profile = Option::from(value);
        self
    }

    pub fn remote(&mut self, value: String) -> &mut InstallArgumentsBuilder {
        self.remote = Option::from(value);
        self
    }

    pub fn settings(&mut self, value: HashMap<String, String>) -> &mut InstallArgumentsBuilder {
        self.settings = value;
        self
    }

    pub fn check_update(&mut self, value: bool) -> &mut InstallArgumentsBuilder {
        self.check_update = value;
        self
    }

    pub fn build(self) -> InstallArguments {
        InstallArguments {
            install_target: self.install_target,
            generators: self.generators,
            install_folder: self.install_folder,
            no_imports: self.no_imports,
            build_configurations: self.build_configurations,
            envs: self.envs,
            options: self.options,
            profile: self.profile,
            remote: self.remote,
            settings: self.settings,
            check_update: self.check_update,
        }
    }
}

#[test]
fn test_install_arguments() {
    let mut builder = InstallArgumentsBuilder::new(
        InstallTarget::ConanFile {
            path: "conanfile.txt".into(),
            reference: None,
        },
        "build".into(),
    );
    builder
        .envs(hashmap!("SomeEnv".into() => "SomeValue".into(),
        "SomeEnv2".into() => "SomeValue2".into()))
        .generators(vec![Generator::JSON])
        .options(hashmap!("SomeOpt".into() => "SomeValue".into(),
        "SomeOpt2".into() => "SomeValue2".into()))
        .settings(hashmap!("SomeSetting".into() => "SomeValue".into(),
        "SomeSetting2".into() => "SomeValue2".into()));
    let arguments = builder.build();
    println!("{:?}", arguments);
    println!("{:?}", arguments.to_commandline_arguements());
}

pub fn create_install_command(
    conan_program: &std::path::Path,
    install_arguments: &InstallArguments,
) -> Command {
    let mut command = Command::new(conan_program);
    command.args(install_arguments.to_commandline_arguements());
    command
}

#[derive(Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub rootpath: String,
    pub sysroot: String,
    pub include_paths: Vec<String>,
    pub lib_paths: Vec<String>,
    pub bin_paths: Vec<String>,
    pub build_paths: Vec<String>,
    pub res_paths: Vec<String>,
    pub libs: Vec<String>,
    pub system_libs: Vec<String>,
    pub defines: Vec<String>,
    pub cflags: Vec<String>,
    pub cxxflags: Vec<String>, // Do not use
    pub sharedlinkflags: Vec<String>,
    pub exelinkflags: Vec<String>,
    pub frameworks: Vec<String>,
    pub framework_paths: Vec<String>,
    pub cppflags: Vec<String>,
}

#[derive(Deserialize)]
pub struct ConanBuildInfo {
    pub deps_env_info: HashMap<String, Vec<String>>,
    pub deps_user_info: HashMap<String, HashMap<String, String>>,
    pub dependencies: Vec<DependencyInfo>,
    pub settings: HashMap<String, String>,
    pub options: HashMap<String, HashMap<String, String>>,
}

impl ConanBuildInfo {
    pub fn create_from_json(json_content: &str) -> ConanBuildInfo {
        serde_json::from_str(json_content).unwrap()
    }
}

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

pub fn cargo_os_to_conan_os(os_name: &str) -> &str {
    CARGO_OS_TO_CONAN_OS.get(os_name).unwrap_or(&os_name)
}

// TODO: Some arch contains endian information
pub fn cargo_arch_to_conan_arch(arch_name: &str) -> &str {
    CARGO_ARCH_TO_CONAN_ARCH
        .get(arch_name)
        .unwrap_or(&arch_name)
}
