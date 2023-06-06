use std::env::set_current_dir;

use bindgen::generate_bindings;
use cli::get_matches;

use anyhow::anyhow;
use helper::workspace_root_dir;
use setup::{add_rust_targets, install_macos_sdk, install_zig_build};
use sharedlibs::generate_shared_libs;
use uniffi_zcash_test::test_data::generate_test_data;
use zcash_proofs::download_sapling_parameters;

mod bindgen;
mod cli;
mod helper;
mod publish;
mod release;
mod setup;
mod sharedlibs;

const PYTHON: &str = "python";
const RUBY: &str = "ruby";
const KOTLIN: &str = "kotlin";
const SWIFT: &str = "swift";

const SUPPORTED_LANGUAGES: [&str; 4] = [PYTHON, RUBY, KOTLIN, SWIFT];

fn main() -> anyhow::Result<()> {
    let matches = get_matches();

    let root_dir = workspace_root_dir()?;
    let bindings_dir = root_dir.join("bindings");
    let packages_dir = root_dir.join("packages");

    set_current_dir(&root_dir)?;

    match matches.subcommand() {
        Some(("setup", args)) => match args.subcommand() {
            Some(("buildenv", _)) => {
                add_rust_targets()?;
                install_zig_build()?;
                Ok(install_macos_sdk()?)
            }
            Some(("saplingparams", _)) => match download_sapling_parameters(None) {
                Ok(paths) => {
                    println!(
                        "Downloaded spend parameters at : {}",
                        paths.spend.to_string_lossy()
                    );
                    println!(
                        "Downloaded output parameters at : {}",
                        paths.output.to_string_lossy()
                    );
                    Ok(())
                }
                Err(err) => Err(anyhow!(err.to_string())),
            },
            Some(("testdata", _)) => {
                generate_test_data(true);
                Ok(())
            }
            _ => Err(anyhow!("Command not found. See help.")),
        },

        Some(("sharedlibs", _)) => Ok(generate_shared_libs(&root_dir)?),
        Some(("bindgen", args)) => {
            let languages: Vec<String> = args
                .try_get_many::<String>("languages")?
                .unwrap()
                .map(Clone::clone)
                .collect();
            Ok(generate_bindings(&root_dir, &languages)?)
        }
        Some(("release", args)) => {
            let package_template_dir = root_dir.join("uniffi-zcash-cli").join("templates");
            match args.subcommand() {
                Some((PYTHON, args)) => {
                    let cfg = release::PythonConfig {
                        version: args.try_get_one::<String>("version")?.unwrap().to_owned(),
                        package_template_dir: package_template_dir.join(PYTHON),
                        test_app_template_dir: package_template_dir.join("python_test_app"),
                        bindings_dir: bindings_dir.join(PYTHON),
                        package_dir: packages_dir.join(PYTHON),
                    };
                    Ok(release::python(&cfg)?)
                }
                Some((RUBY, args)) => {
                    let cfg = release::RubyConfig {
                        version: args.try_get_one::<String>("version")?.unwrap().to_owned(),
                        package_template_dir: package_template_dir.join(RUBY),
                        test_app_template_dir: package_template_dir.join("ruby_test_app"),
                        bindings_dir: bindings_dir.join(RUBY),
                        package_dir: packages_dir.join(RUBY),
                    };
                    Ok(release::ruby(&cfg)?)
                }
                Some((KOTLIN, args)) => {
                    let cfg = release::KotlinConfig {
                        version: args.try_get_one::<String>("version")?.unwrap().to_owned(),
                        package_template_dir: package_template_dir.join(KOTLIN),
                        test_app_template_dir: package_template_dir.join("kotlin_test_app"),
                        bindings_dir: bindings_dir.join(KOTLIN),
                        package_dir: packages_dir.join(KOTLIN),
                    };
                    Ok(release::kotlin(&cfg)?)
                }
                Some((SWIFT, args)) => {
                    let cfg = release::SwiftConfig {
                        version: args.try_get_one::<String>("version")?.unwrap().to_owned(),
                        git_repo_url: args
                            .try_get_one::<String>("git_repo_url")?
                            .unwrap()
                            .to_owned(),
                        package_template_dir: package_template_dir.join(SWIFT),
                        test_app_template_dir: package_template_dir.join("swift_test_app"),
                        bindings_dir: bindings_dir.join(SWIFT),
                        package_dir: packages_dir.join(SWIFT),
                    };
                    Ok(release::swift(&cfg)?)
                }
                _ => Err(anyhow!("Command not found. See help.")),
            }
        }
        Some(("publish", args)) => match args.subcommand() {
            Some((PYTHON, args)) => {
                let cfg = publish::PythonConfig {
                    lang_package_path: packages_dir.join(PYTHON),
                    registry_url: args
                        .try_get_one::<String>("registry_url")?
                        .unwrap()
                        .to_owned(),
                    registry_username: args
                        .try_get_one::<String>("registry_username")?
                        .unwrap()
                        .to_owned(),
                    registry_password: args
                        .try_get_one::<String>("registry_password")?
                        .unwrap()
                        .to_owned(),
                };
                cfg.lang_package_path.try_exists()?;
                Ok(publish::python(&cfg)?)
            }
            Some((RUBY, args)) => {
                let cfg = publish::RubyConfig {
                    lang_package_path: packages_dir.join(RUBY),
                    version: args.try_get_one::<String>("version")?.unwrap().to_owned(),
                    registry_url: args
                        .try_get_one::<String>("registry_url")?
                        .unwrap()
                        .to_owned(),
                    registry_token: args
                        .try_get_one::<String>("registry_token")?
                        .unwrap()
                        .to_owned(),
                };
                cfg.lang_package_path.try_exists()?;
                Ok(publish::ruby(&cfg)?)
            }
            Some((KOTLIN, args)) => {
                let cfg = publish::KotlinConfig {
                    lang_package_path: packages_dir.join(KOTLIN),
                    registry_url: args
                        .try_get_one::<String>("registry_url")?
                        .unwrap()
                        .to_owned(),
                    registry_username: args
                        .try_get_one::<String>("registry_username")?
                        .unwrap()
                        .to_owned(),
                    registry_password: args
                        .try_get_one::<String>("registry_password")?
                        .unwrap()
                        .to_owned(),
                };
                cfg.lang_package_path.try_exists()?;
                Ok(publish::kotlin(&cfg)?)
            }
            Some((SWIFT, args)) => match args.subcommand() {
                Some(("git-repo", args)) => {
                    let cfg = publish::SwiftRepoConfig {
                        lang_package_path: packages_dir.join(SWIFT),
                        git_repo_url: args
                            .try_get_one::<String>("git_repo_url")?
                            .unwrap()
                            .to_owned(),
                    };
                    cfg.lang_package_path.try_exists()?;
                    Ok(publish::swift_repo(&cfg)?)
                }
                Some(("registry", args)) => {
                    let cfg = publish::SwiftRegistryConfig {
                        lang_package_path: packages_dir.join(SWIFT),
                        version: args.try_get_one::<String>("version")?.unwrap().to_owned(),
                        registry_url: args
                            .try_get_one::<String>("registry_url")?
                            .unwrap()
                            .to_owned(),
                        registry_token: args
                            .try_get_one::<String>("registry_token")?
                            .unwrap()
                            .to_owned(),
                    };
                    cfg.lang_package_path.try_exists()?;
                    Ok(publish::swift_registry(&cfg)?)
                }
                _ => Err(anyhow!("Command not found. See help.")),
            },
            _ => Err(anyhow!("Command not found. See help.")),
        },
        _ => Err(anyhow!("Command not found. See help.")),
    }
}
