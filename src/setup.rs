use std::env;
use std::fs::File;
use std::io::Read;

use structopt::StructOpt;

use config_file_model::ConfigurationFile;
use sonar_api::PasswordProvider;
use sonar_api::SonarApi;
use sonar_api_model::{SonarAdminUpdateRequest, SonarGroupCreationRequest, SonarProperty, SonarUser};

#[derive(StructOpt, Debug)]
pub struct SetupCmd {
    #[structopt(name = "file", short = "f", help = "file to the YAML configuration file")]
    file: String,

    #[structopt(name = "sonarUrl", short = "s", help = "URL of SonarQube")]
    url: String,

    #[structopt(name = "adminPassword", short = "p", help = "The password of the administrator; auto-detected with the env. variable: ADMIN_PASSWORD and fallback with default admin password")]
    admin_password: Option<String>,

    #[structopt(name = "nbAttempts", short = "a", default_value = "600", help = "Number of attempts to connect to the API (1sec between attempts)")]
    number_attempts: usize,
}

pub fn setup(cmd: SetupCmd) -> Result<(), reqwest::Error> {
    let config_file: ConfigurationFile = load_configuration(&cmd);
    let admin_modification = config_file.admin.map(|admin| SonarAdminUpdateRequest::from_configuration(&admin, resolve_variables));

    let mut sonar_api = SonarApi::new(cmd.url.to_string(), ::ADMIN_USER.to_string(), init_password_provider(&cmd, &admin_modification), cmd.number_attempts);

    for property in SonarProperty::from_configuration_properties(&config_file.properties.unwrap_or(Vec::new()), resolve_variables) {
        sonar_api.set_property(&property);
    }

    for group in SonarGroupCreationRequest::from_configuration_groups(&config_file.groups.unwrap_or(Vec::new()), resolve_variables) {
        sonar_api.create_group(&group);
    }

    for user in SonarUser::from_configuration_users(&config_file.users.unwrap_or(Vec::new()), resolve_variables) {
        sonar_api.create_user(&user);
    }

    if admin_modification.as_ref().is_some() {
        sonar_api.update_admin(&admin_modification.unwrap());
    }

    Ok(())
}

fn load_configuration(setup: &SetupCmd) -> ConfigurationFile {
    debug!("Run setup command with configuration file {:?}", setup.file);

    let mut file = File::open(&setup.file)
        .expect("Cannot open configuration file.");

    let mut file_content = String::new();
    file.read_to_string(&mut file_content)
        .expect("Cannot read configuration file.");

    return serde_yaml::from_str(&file_content)
        .expect("Error while loading configuration file.");
}

fn resolve_variables(tokenized: &String) -> String {
    let context = std::env::vars().collect();

    let resolve_string = envsubst::substitute(tokenized, &context).unwrap();

    assert!(!envsubst::is_templated(&resolve_string), format!("Cannot resolve all variables from [{}].", tokenized));

    return resolve_string;
}

fn init_password_provider(_cmd: &SetupCmd, mut _admin_modification: &Option<SonarAdminUpdateRequest>) -> PasswordProvider {
    let target_password = _admin_modification.as_ref()
        .and_then(|modification| modification.password.as_ref())
        .map(|password| password.to_string());

    let current_password = _cmd.admin_password.as_ref()
        .or(env::var(::ENV_ADMIN_PASSWORD.to_string()).ok().as_ref())
        .or(Some(::DEFAULT_ADMIN_PASSWORD.to_string()).as_ref())
        .map(|pwd| pwd.to_string());

    return PasswordProvider::password_or_fallback(target_password, current_password);
}
