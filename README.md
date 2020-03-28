# Sonar Configuration as Code

![Rust](https://github.com/sebge2/sonar-config-as-code/workflows/Rust/badge.svg)

This command tool ease the configuration of SonarQube. 

The following properties can be defined in an YAML file:
* groups,
* users,
* group permissions,
* and properties.

For an example, please go in _./example/sonar.yaml_.

Once configured, just run the command tool:
````
sonar-as-code setup -f ./example/sonar.yaml -s http://192.168.99.100:31973/ -u admin -p myPassword 
````

It is also possible to generate a token for the current user:
````
sonar-as-code setup -f ./example/sonar.yaml -s http://192.168.99.100:31973/ -u admin -p myPassword -n myTokenName
````

The command tool is also available in a docker image: sebge2/sonar-as-code:$VERSION.


## Links

* [Static App](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html)
