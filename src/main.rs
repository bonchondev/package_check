use std::process::Command;

use actix_cors::Cors;
use actix_web::{get, http, post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct PackageRequest {
    kind: String,
    packages: String,
}

#[derive(Debug)]
enum PackageType {
    Python,
    R,
    NotImplemented,
}

#[derive(Debug)]
enum SubPackageType {
    Base,
    Bioconductor,
    Devtools,
    Remote,
    NotExist,
}

impl SubPackageType {
    fn subcommand(&self) -> &str {
        match self {
            Self::Base => "install.packages",
            Self::Remote => "remote::install_github",
            Self::Bioconductor => "BiocManager::install",
            Self::Devtools => "devtools::install_github",
            Self::NotExist => "",
        }
    }
}

#[derive(Debug)]
struct Package {
    command: PackageType,
    args: Vec<String>,
    install_command: SubPackageType,
}

impl Package {
    fn run_command(&self) -> Command {
        let cmd = match self.command {
            PackageType::Python => "pip".to_string(),
            PackageType::NotImplemented => "echo".to_string(),
            PackageType::R => "R".to_string(),
        };
        let args = match self.command {
            PackageType::R => {
                let parsed = &self
                    .args
                    .iter()
                    .map(|pkg| format!("'{}'", pkg))
                    .collect::<Vec<_>>()
                    .join(",");
                vec![
                    "--no-save".to_string(),
                    "-e".to_string(),
                    format!("{}(c({}))", self.install_command.subcommand(), parsed),
                ]
            }
            _ => {
                let mut install_args = vec!["install".to_string()];
                install_args.extend(self.args.clone());
                install_args
            }
        };
        let mut output = Command::new(cmd);
        output.args(args);
        output
    }
}

fn package_formatter(pkgs: String, pkg_type: String) -> Package {
    let packages = pkgs
        .split(",")
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
    let commands = match pkg_type.as_str() {
        "pip" => Package {
            command: PackageType::Python,
            args: packages,
            install_command: SubPackageType::NotExist,
        },
        "base" => Package {
            command: PackageType::R,
            args: packages,
            install_command: SubPackageType::Base,
        },
        "remote" => Package {
            command: PackageType::R,
            args: packages,
            install_command: SubPackageType::Remote,
        },
        "devtools" => Package {
            command: PackageType::R,
            args: packages,
            install_command: SubPackageType::Devtools,
        },
        "bioconductor" => Package {
            command: PackageType::R,
            args: packages,
            install_command: SubPackageType::Bioconductor,
        },
        _ => Package {
            command: PackageType::NotImplemented,
            args: packages,
            install_command: SubPackageType::NotExist,
        },
    };
    commands
}

#[get("/")]
async fn index() -> impl Responder {
    "Ok."
}

#[post("/install")]
async fn installation(form: web::Json<PackageRequest>) -> HttpResponse {
    let data = package_formatter(form.packages.clone(), form.kind.clone());
    let mut cmd = data.run_command();
    let status = cmd.status().expect("huh??");
    HttpResponse::Ok().body(format!("{}", status))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new().wrap(cors).service(index).service(installation)
    })
    .bind(("0.0.0.0", 9000))?
    .run()
    .await
}
