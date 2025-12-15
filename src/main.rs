use std::time::Duration;

use bowser::{Args, Job, WebPage, contruct_proxy, pfsense, read_job_file};
use clap::Parser;
use thirtyfour::prelude::*;

const PFSENSE_MODULE: &'static str = "pfsense";

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::chrome();
    let _ = caps.add_arg("--disable-blink-features=AutomationControlled");
    let _ = caps.add_arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36");

    let args = Args::parse();

    match args.proxy {
        Some(proxy) => {
            println!("Using proxy-server {}", proxy);
            let proxy_config = contruct_proxy(proxy);

            caps.set_proxy(proxy_config.clone())
                .expect("Unable to set proxy-server")
        }
        _ => println!("No proxy passed"),
    }

    if args.insecure {
        caps.accept_insecure_certs(true)
            .expect("Browser cannot be set to insecure mode")
    }

    let driver = WebDriver::new(args.webdriver_server, caps).await?;
    let webpage = WebPage::new(args.url, driver);

    let module = match args.module {
        Some(a) => a,
        _ => "".to_string(),
    };
    if module == PFSENSE_MODULE {
        {
            pfsense::login(
                args.username.unwrap_or("no".to_string()),
                args.password.unwrap_or("no".to_string()),
                &webpage,
            )
            .await;

            webpage.driver.get(&webpage.url).await.unwrap();

            match args.alias_file {
                Some(file) => {
                    let aliases = pfsense::load_alias_file(file);
                    for alias in aliases {
                        pfsense::add_host_aliases(&webpage, alias).await;
                    }
                }
                _ => println!("No alias file"),
            };

            match args.dns_forwarder_file {
                Some(file) => {
                    let dns_forwarders = pfsense::load_dns_forwarder_file(file);
                    for record in dns_forwarders {
                        pfsense::add_dns_forwarder(&webpage, &record).await;
                    }
                }
                _ => println!("No dns file"),
            };
        };
    }

    let tasks = {
        match args.job_file_path {
            Some(job_file) => read_job_file(&job_file),
            _ => {
                webpage.driver.quit().await?;
                return Ok(());
            }
        }
    };

    let job = Job { tasks: tasks };

    webpage.run_job(job).await;
    tokio::time::sleep(Duration::from_secs(10)).await;
    webpage.driver.quit().await?;
    Ok(())
}
