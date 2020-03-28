use structopt::StructOpt;

use sonar_api::SonarApi;

#[derive(StructOpt, Debug)]
pub struct GenerateTokenCmd {
    #[structopt(name = "name", short = "name", about = "name of the generated token")]
    name: String,

    #[structopt(name = "sonarUrl", short = "s", about = "URL of SonarQube")]
    url: String,

    #[structopt(name = "username", short = "u", default_value = "admin", about = "Username of the administrator")]
    username: String,

    #[structopt(name = "password", short = "p", default_value = "admin", about = "Password of the administrator")]
    password: String,

    #[structopt(name = "nbAttempts", short = "a", default_value = "600", about = "Number of attemps to connect to the API (1sec between attempts)")]
    number_attempts: usize,
}

pub fn generate_token(cmd: GenerateTokenCmd) -> Result<(), reqwest::Error> {
    let sonar_api = SonarApi::new(cmd.url.to_string(), cmd.username.to_string(), cmd.password.to_string(), cmd.number_attempts);

    sonar_api.wait_ready();

    println!("{}", sonar_api.generate_user_token(&cmd.username.to_string(), &cmd.name.to_string()));

    Ok(())
}
