use structopt::StructOpt;

use sonar_api::PasswordProvider;
use sonar_api::SonarApi;

#[derive(StructOpt, Debug)]
pub struct GenerateTokenCmd {
    #[structopt(name = "name", short = "name", help = "name of the generated token")]
    name: String,

    #[structopt(name = "sonarUrl", short = "s", help = "URL of SonarQube")]
    url: String,

    #[structopt(name = "username", short = "u", default_value = "admin", help = "Username")]
    username: String,

    #[structopt(name = "password", short = "p", default_value = "admin", help = "User password")]
    password: String,

    #[structopt(name = "nbAttempts", short = "a", default_value = "600", help = "Number of attempts to connect to the API (1sec between attempts)")]
    number_attempts: usize,
}

pub fn generate_token(cmd: GenerateTokenCmd) -> Result<(), reqwest::Error> {
    let sonar_api =
        SonarApi::new(cmd.url.to_string(), cmd.username.to_string(), PasswordProvider::specific_password(cmd.password), cmd.number_attempts);

    println!("{}", sonar_api.generate_user_token(&cmd.username.to_string(), &cmd.name.to_string()));

    Ok(())
}
