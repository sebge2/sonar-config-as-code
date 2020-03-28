use std::fs::File;
use std::io::Read;

use structopt::StructOpt;

use config_file_model::ConfigurationFile;
use sonar_api::SonarApi;
use sonar_api_model::{SonarGroupCreationRequest, SonarProperty, SonarUser};

#[derive(StructOpt, Debug)]
pub struct SetupCmd {
    #[structopt(name = "file", short = "f", about = "file to the YAML configuration file")]
    file: String,

    #[structopt(name = "sonarUrl", short = "s", about = "URL of SonarQube")]
    url: String,

    #[structopt(name = "username", short = "u", default_value = "admin", about = "Username of the administrator")]
    username: String,

    #[structopt(name = "password", short = "p", default_value = "admin", about = "Password of the administrator")]
    password: String,

    #[structopt(name = "nbAttempts", short = "a", default_value = "600", about = "Number of attemps to connect to the API (1sec between attempts)")]
    number_attempts: usize,
}

    pub fn setup(cmd: SetupCmd) -> Result<(), reqwest::Error> {
    let config_file: ConfigurationFile = load_configuration(&cmd);

    let mut sonar_api = SonarApi::new(cmd.url.to_string(), cmd.username.to_string(), cmd.password.to_string(), cmd.number_attempts);

    sonar_api.wait_ready();

    for property in SonarProperty::from_configuration_properties(&config_file.properties.unwrap_or(Vec::new()), resolve_variables) {
        sonar_api.set_property(&property);
    }

    for group in SonarGroupCreationRequest::from_configuration_groups(&config_file.groups.unwrap_or(Vec::new()), resolve_variables) {
        sonar_api.create_group(&group);
    }

    for user in SonarUser::from_configuration_users(&config_file.users.unwrap_or(Vec::new()), resolve_variables) {
        sonar_api.create_user(&user);
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
