use std::{fs::File, io::BufReader, time::Duration};

use clap::Parser;
use serde::{Deserialize, Serialize};
use thirtyfour::{ChromeCapabilities, Proxy, prelude::*};
pub mod pfsense;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub job_file_path: Option<String>,
    #[arg(short, long)]
    pub webdriver_server: String,
    #[arg(short, long)]
    pub url: String,
    #[arg(long)]
    pub proxy: Option<String>,
    #[arg(long)]
    pub username: Option<String>,
    #[arg(long)]
    pub password: Option<String>,
    #[arg(long)]
    pub module: Option<String>,
    #[arg(long)]
    pub alias_file: Option<String>,
    #[arg(long)]
    pub dns_forwarder_file: Option<String>,
    #[arg(short, long, default_value_t = false)]
    pub insecure: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ActionOptions {
    SendKeys,
    Click,
    GetValue,
    NoAction,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ByOptions {
    ID,
    CSS,
    NAME,
    TAG,
    XPATH,
}

pub struct Job {
    pub tasks: Vec<Task>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub by: ByOptions,
    pub id: String,
    pub value: String,
    pub action: ActionOptions,
    pub path: String,
}

impl Task {
    pub fn new(
        by: ByOptions,
        id: String,
        value: String,
        action: ActionOptions,
        path: String,
    ) -> Task {
        Task {
            by,
            id,
            value,
            action,
            path,
        }
    }

    pub fn defalt() -> Vec<Task> {
        let task = Task {
            by: ByOptions::ID,
            id: "".to_string(),
            value: "".to_string(),
            action: ActionOptions::NoAction,
            path: "".to_string(),
        };
        vec![task]
    }
}

pub struct WebPage {
    pub url: String,
    pub driver: WebDriver,
}

impl WebPage {
    pub fn new(url: String, driver: WebDriver) -> WebPage {
        WebPage {
            url: url,
            driver: driver,
        }
    }
    pub async fn run_job(&self, job: Job) {
        let url = &self.url;
        self.driver
            .goto(format!("{url}"))
            .await
            .expect("Unable to goto url");
        for task in job.tasks {
            let current_url = self
                .driver
                .current_url()
                .await
                .expect("Could not get current url")
                .to_string();

            let target_path = task.path;
            let target_url = format!("{url}{target_path}");

            if !target_path.is_empty() {
                match target_url {
                    target_url if target_url == current_url => {
                        println!("Current url is Target url")
                    }
                    _ => self.driver.goto(target_url).await.unwrap(),
                }
            }
            match task.action {
                ActionOptions::NoAction => return (),
                _ => println!("Action"),
            }

            println!("Running: {}", task.id);
            let element = match task.by {
                ByOptions::ID => self
                    .driver
                    .find(By::Id(task.id))
                    .await
                    .expect("Failed to find element by ID"),
                ByOptions::CSS => self
                    .driver
                    .find(By::Css(task.id))
                    .await
                    .expect("Failed to find element by CSS"),
                ByOptions::NAME => self
                    .driver
                    .find(By::Name(task.id))
                    .await
                    .expect("Failed to find element by Name"),
                ByOptions::XPATH => self
                    .driver
                    .find(By::XPath(task.id))
                    .await
                    .expect("Failed to find element by XPATH"),
                ByOptions::TAG => self
                    .driver
                    .find(By::Tag(task.id))
                    .await
                    .expect("Failed to find element by ID"),
            };
            match task.action {
                ActionOptions::SendKeys => element.send_keys(task.value).await.unwrap(),
                ActionOptions::Click => element.click().await.unwrap(),
                ActionOptions::GetValue => {
                    println!("{}", element.inner_html().await.unwrap().to_string())
                }
                ActionOptions::NoAction => println!("No action"),
            };
        }
    }
}

pub fn read_job_file(file_path: &str) -> Vec<Task> {
    let file = File::open(file_path).expect("Unable to read file");
    let reader = BufReader::new(file);

    serde_json::from_reader(reader).expect("Unable to read file")
}

pub fn contruct_proxy(proxy: String) -> Proxy {
    let proxy_config = Proxy::Manual {
        ftp_proxy: None,
        http_proxy: Some(proxy.clone()),
        ssl_proxy: Some(proxy.clone()),
        socks_proxy: None,
        socks_version: None,
        socks_username: None,
        socks_password: None,
        no_proxy: None,
    };
    proxy_config
}
