#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;

use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
use std::string::String;

#[cfg(feature = "cargo")]
pub mod cargo;

pub struct Conan {
    pub path: std::path::PathBuf,
}

impl Conan {
    pub fn new(conan_path: std::path::PathBuf) -> Conan {
        Conan { path: conan_path }
    }

    pub fn find_system_conan() -> Option<Conan> {
        if let Ok(conan_path) = which::which("conan") {
            return Some(Conan { path: conan_path });
        }

        None
    }
}

lazy_static! {
    static ref CONAN_VERSION_REGEX: Regex = Regex::new("Conan version ([\\d\\.]+)").unwrap();
    static ref CONAN_REMOTE_REGEX: Regex = Regex::new("(\\S+) (\\S+) (True|False)").unwrap();
}

#[derive(Debug)]
pub struct Remote {
    pub name: String,
    pub url: String,
    pub verify_ssl: Option<bool>,
}

impl Remote {
    pub fn new(name: String, url: String) -> Remote {
        Remote {
            name,
            url,
            verify_ssl: None,
        }
    }
}

impl Conan {
    pub fn determine_version(&self) -> Option<String> {
        let output = Command::new(&self.path)
            .arg("--version")
            .output()
            .expect(&format!(
                "Cannot execute conan from path \"{:?}\"",
                &self.path
            ));
        let output_string = String::from_utf8(output.stdout).ok()?;
        let found_version = CONAN_VERSION_REGEX.captures(&output_string)?.get(1)?;
        Some(found_version.as_str().into())
    }

    pub fn get_remote_list(&self) -> Option<Vec<Remote>> {
        let output = Command::new(&self.path)
            .arg("remote")
            .arg("list")
            .arg("--raw")
            .output()
            .expect(&format!(
                "Cannot execute conan from path \"{:?}\"",
                &self.path
            ));

        let output_string = String::from_utf8(output.stdout).ok()?;
        let mut result = Vec::new();

        for line in output_string.lines() {
            let matched_remote = CONAN_REMOTE_REGEX.captures(&line)?;
            result.push(Remote {
                name: matched_remote.get(1)?.as_str().into(),
                url: matched_remote.get(2)?.as_str().into(),
                verify_ssl: Some(matched_remote.get(3)?.as_str() == "True"),
            });
        }

        Some(result)
    }

    pub fn add_remote(&self, remote: &Remote, index: Option<u32>, force: bool) -> bool {
        let mut command = Command::new(&self.path);
        let mut arguments = vec!["remote".to_owned(), "add".to_owned()];
        if let Some(index) = index {
            arguments.push("-i".into());
            arguments.push(index.to_string());
        }
        if force {
            arguments.push("--force".into());
        }
        arguments.push(remote.name.clone());
        arguments.push(remote.url.clone());

        match remote.verify_ssl {
            Some(true) => {
                arguments.push("True".into());
            }
            Some(false) => {
                arguments.push("False".into());
            }
            None => {}
        }

        command.spawn().unwrap().wait().is_ok()
    }

    pub fn remove_remote(&self, remote_name: &str) -> bool {
        Command::new(&self.path)
            .arg("remote")
            .arg("remove")
            .arg(remote_name)
            .spawn()
            .unwrap()
            .wait()
            .is_ok()
    }
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
    pub envs_build: HashMap<String, String>,
    pub options: HashMap<String, String>,
    pub options_build: HashMap<String, String>,
    pub profile: Option<String>,
    pub profile_build: Option<String>,
    pub remote: Option<String>,
    pub settings: HashMap<String, String>,
    pub settings_build: HashMap<String, String>,
    pub check_update: bool,
}

impl InstallArguments {
    pub fn to_commandline_arguments(&self) -> Vec<String> {
        let mut result = vec!["install".into()];

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

        for (env_key, env_value) in &self.envs_build {
            result.push("-e:b".into());
            result.push(format!("{}={}", env_key, env_value));
        }

        for (option_key, option_value) in &self.options {
            result.push("-o".into());
            result.push(format!("{}={}", option_key, option_value));
        }

        for (option_key, option_value) in &self.options_build {
            result.push("-o:b".into());
            result.push(format!("{}={}", option_key, option_value));
        }

        if let Some(profile) = &self.profile {
            result.push("-pr".into());
            result.push(profile.clone());
        }

        if let Some(profile_build) = &self.profile_build {
            result.push("-pr:b".into());
            result.push(profile_build.clone());
        }

        if let Some(remote) = &self.remote {
            result.push("-r".into());
            result.push(remote.clone());
        }

        for (setting_key, setting_value) in &self.settings {
            result.push("-s".into());
            result.push(format!("{}={}", setting_key, setting_value));
        }

        for (setting_key, setting_value) in &self.settings_build {
            result.push("-s:b".into());
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
    envs_build: HashMap<String, String>,
    options: HashMap<String, String>,
    options_build: HashMap<String, String>,
    profile: Option<String>,
    profile_build: Option<String>,
    remote: Option<String>,
    settings: HashMap<String, String>,
    settings_build: HashMap<String, String>,
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
            envs_build: HashMap::new(),
            options: HashMap::new(),
            options_build: HashMap::new(),
            profile: None,
            profile_build: None,
            remote: None,
            settings: HashMap::new(),
            settings_build: HashMap::new(),
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

    pub fn envs_build(&mut self, value: HashMap<String, String>) -> &mut InstallArgumentsBuilder {
        self.envs_build = value;
        self
    }

    pub fn options(&mut self, value: HashMap<String, String>) -> &mut InstallArgumentsBuilder {
        self.options = value;
        self
    }

    pub fn options_build(&mut self, value: HashMap<String, String>) -> &mut InstallArgumentsBuilder {
        self.options_build = value;
        self
    }

    pub fn profile(&mut self, value: String) -> &mut InstallArgumentsBuilder {
        self.profile = Some(value);
        self
    }

    pub fn profile_build(&mut self, value: String) -> &mut InstallArgumentsBuilder {
        self.profile_build = Some(value);
        self
    }

    pub fn remote(&mut self, value: String) -> &mut InstallArgumentsBuilder {
        self.remote = Some(value);
        self
    }

    pub fn settings(&mut self, value: HashMap<String, String>) -> &mut InstallArgumentsBuilder {
        self.settings = value;
        self
    }

    pub fn settings_build(&mut self, value: HashMap<String, String>) -> &mut InstallArgumentsBuilder {
        self.settings_build = value;
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
            envs_build: self.envs_build,
            options: self.options,
            options_build: self.options_build,
            profile: self.profile,
            profile_build: self.profile_build,
            remote: self.remote,
            settings: self.settings,
            settings_build: self.settings_build,
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
        .envs(hashmap!(
            "SomeEnv".into() => "SomeValue".into(),
            "SomeEnv2".into() => "SomeValue2".into()
        ))
        .envs_build(hashmap!(
            "SomeBuildEnv".into() => "SomeBuildValue".into(),
            "SomeBuildEnv2".into() => "SomeBuildValue2".into()
        ))
        .generators(vec![Generator::JSON])
        .options(hashmap!(
            "SomeOpt".into() => "SomeValue".into(),
            "SomeOpt2".into() => "SomeValue2".into()
        ))
        .options_build(hashmap!(
            "SomeBuildOpt".into() => "SomeBuildValue".into(),
            "SomeBuildOpt2".into() => "SomeBuildValue2".into()
        ))
        .settings(hashmap!(
            "SomeSetting".into() => "SomeValue".into(),
            "SomeSetting2".into() => "SomeValue2".into()
        ))
        .settings_build(hashmap!(
            "SomeBuildSetting".into() => "SomeBuildValue".into(),
            "SomeBuildSetting2".into() => "SomeBuildValue2".into()
        ))
        .profile("android".into())
        .profile_build("gcc".into());
    let arguments = builder.build();
    println!("{:?}", arguments);
    println!("{:?}", arguments.to_commandline_arguments());
}

impl Conan {
    pub fn create_install_command(&self, install_arguments: &InstallArguments) -> Command {
        let mut command = Command::new(&self.path);
        command.args(install_arguments.to_commandline_arguments());
        command
    }
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct ConanBuildInfo {
    pub deps_env_info: HashMap<String, Vec<String>>,
    pub deps_user_info: HashMap<String, HashMap<String, String>>,
    pub dependencies: Vec<DependencyInfo>,
    pub settings: HashMap<String, String>,
    pub options: HashMap<String, HashMap<String, String>>,
}

impl ConanBuildInfo {
    pub fn create_from_json(json_content: &str) -> Option<ConanBuildInfo> {
        serde_json::from_str(json_content).ok()
    }

    pub fn create_from_json_reader(reader: impl std::io::Read) -> Option<ConanBuildInfo> {
        serde_json::from_reader(reader).ok()
    }

    pub fn find_dependency(&self, name: &str) -> Option<&DependencyInfo> {
        self.dependencies.iter().find(|d| d.name == name)
    }
}

#[test]
fn test_install() {
    let conan = Conan::find_system_conan().unwrap();
    let mut builder = InstallArgumentsBuilder::new(
        InstallTarget::Package {
            reference: "zlib/1.2.11@_/_".into(),
        },
        "temp".into(),
    );
    builder
        .build_configurations(vec![BuildConfiguration::Missing])
        .generators(vec![Generator::JSON]);
    conan
        .create_install_command(&builder.build())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    let build_info_file = std::fs::File::open("temp/conanbuildinfo.json").unwrap();
    let build_info =
        ConanBuildInfo::create_from_json_reader(std::io::BufReader::new(build_info_file)).unwrap();
    println!("{:?}", build_info);
    let zlib = build_info.find_dependency("zlib").unwrap();
    println!("{:?}", zlib);
}
