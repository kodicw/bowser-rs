use std::{fmt::format, fs, time::Duration};

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use table_extract::Table;
use thirtyfour::By;
use tokio::time::sleep;

use crate::{ActionOptions, ByOptions, Job, Task, WebPage};

// pub enum Paths {
//     ALIASES = "/firewall_aliases_edit.php",
//     DNS_FORWARDERS = "/services_dnsmasq.php",
// }

pub enum AliasType {
    // HOST = "host",
    // NETWORKS = "networks",
    // PORTS = "port",
    // URL = "url",
    // URLPORTS = "url_ports",
    // URLTABLE = "urltable",
    // URLTBLEPORTS = "urltable_ports",
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Alias {
    pub name: String,
    pub description: String,
    pub type_: String,
    pub entrys: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DNSForwarder {
    domain: String,
    ip: String,
    source_ip: Option<String>,
    description: String,
}

pub async fn login(username: String, password: String, web: &WebPage) {
    let page = web.driver.get(format!("{}", web.url)).await.unwrap();
    let username_field = match web.driver.find(By::Id("usernamefld")).await {
        Ok(el) => el,
        _ => panic!("Failed to find username field"),
    };
    let password_field = match web.driver.find(By::Id("passwordfld")).await {
        Ok(el) => el,
        _ => panic!("Failed to find password field"),
    };

    let login_button = match web.driver.find(By::Name("login")).await {
        Ok(e) => e,
        _ => panic!("Failed to find login button"),
    };

    println!("logging in");
    username_field.send_keys(username).await.unwrap();
    password_field.send_keys(password).await.unwrap();
    login_button.click().await.unwrap();
}

pub async fn add_dns_forwarder(web: &WebPage, forwarder: &DNSForwarder) {
    let url = &web.url;
    let edit_path = "/services_dnsmasq_domainoverride_edit.php";
    let path = "/services_dnsmasq.php";
    web.driver
        .get(format!("{url}{path}"))
        .await
        .expect("Failed to navigate to DNS forwarder settings page");
    let table_html = web
        .driver
        .find(By::XPath("/html/body/div/div[2]"))
        .await
        .expect("Could not find table HTML");
    let html_raw = table_html.outer_html().await.unwrap();

    if let Some(table) =
        Table::find_by_headers(&html_raw, &["Domain", "IP", "Description", "Actions"])
    {
        let mut current_dns_forwarders: Vec<&str> = vec![];
        for row in &table {
            if let Some(domain) = row.get("Domain") {
                current_dns_forwarders.push(domain);
            };
        }
        if !current_dns_forwarders.contains(&forwarder.domain.as_str()) {
            println!("Adding {} to dns forwarders", forwarder.domain);
            web.driver.get(format!("{url}{edit_path}")).await.unwrap();
            let domain_field = web
                .driver
                .find(By::Id("domain"))
                .await
                .expect("Could not find Domain field");
            let ip_field = web
                .driver
                .find(By::Id("ip"))
                .await
                .expect("Could not find ip field");
            let description_field = web
                .driver
                .find(By::Id("descr"))
                .await
                .expect("Could not find description field");
            let save_button = web
                .driver
                .find(By::Id("save"))
                .await
                .expect("Unable to find save button");

            domain_field.send_keys(&forwarder.domain).await.unwrap();
            ip_field.send_keys(&forwarder.ip).await.unwrap();
            description_field
                .send_keys(&forwarder.description)
                .await
                .unwrap();
            save_button.click().await.unwrap()
        } else {
            println!("{} dns forwarders already exists", forwarder.domain)
        }
    };
}

pub async fn add_host_aliases(web: &WebPage, alias: Alias) {
    web.driver
        .get(format!("{}//firewall_aliases.php?tab=ip", web.url))
        .await
        .unwrap();

    let html_raw = web
        .driver
        .page_source()
        .await
        .expect("Unable to extract html from IP alias page");

    let mut url = format!("{}/{}", web.url, "/firewall_aliases_edit.php");

    if let Some(table) = table_extract::Table::find_first(&html_raw) {
        for row in &table {
            if let Some(name) = row.get("Name") {
                if let Some(t) = row.get("Actions") {
                    if alias.name == name {
                        println!("{:?}", name);
                        let fragment = Html::parse_fragment(&t);

                        let selector = Selector::parse(r#"a[title="Edit alias"]"#)
                            .expect("Failed to create selector");

                        if let Some(target_href) = fragment
                            .select(&selector)
                            .filter_map(|element| element.value().attr("href"))
                            .next()
                        {
                            println!("Updating url for current existing alias {}", name);
                            url = format!("{}/{}", web.url, target_href);
                        };
                    }
                };
            };
        }
    };

    let name_field = Task::new(
        ByOptions::ID,
        "name".to_string(),
        alias.name.clone(),
        ActionOptions::SendKeys,
        "".to_string(),
    );

    let description_field = Task::new(
        ByOptions::ID,
        "descr".to_string(),
        alias.description.clone(),
        ActionOptions::SendKeys,
        "".to_string(),
    );

    let tasks: Vec<Task> = vec![name_field];
    let job: Job = Job { tasks: tasks };

    web.driver.get(url).await.unwrap();

    let current_ips_fdqn = match web
        .driver
        .find_all(By::XPath("//*[starts-with(@id, 'address') and not(contains(@id, '_')) and translate(substring(@id, string-length('address') + 1), '0123456789', '') = '']"))
        .await
    {
        Ok(elements) => elements,
        _ => panic!("Unable to find aliases"),
    };
    let mut c_ip_fdqn: Vec<String> = vec![];

    for ip_fdqn in current_ips_fdqn {
        let v = match ip_fdqn.get_property("value").await {
            Ok(v) => match v {
                Some(v) => v,
                _ => panic!("No value for attribute value"),
            },
            _ => panic!("Could not get value from ip list"),
        };
        c_ip_fdqn.push(v);
    }
    let mut current_entry = c_ip_fdqn.len();

    let mut need_to_save = false;

    for ip_fdqn in alias.entrys {
        if !c_ip_fdqn.contains(&ip_fdqn) {
            let add_row_button = web
                .driver
                .find(By::Id("addrow"))
                .await
                .expect("Failed to find addrow button");
            let current_row_id = format!("address{}", current_entry);
            add_row_button.click().await.expect("");
            let current_row = web
                .driver
                .find(By::Id(current_row_id))
                .await
                .expect("Failed to find row");
            println!("Adding {} to alias", &ip_fdqn);
            current_row.send_keys(&ip_fdqn).await.expect("");
            current_entry += 1;
            need_to_save = true;
        } else {
            println!("IP/FQDN: {} already in {}", ip_fdqn, &alias.name)
        }
    }
    if need_to_save {
        if let Ok(save_button) = web.driver.find(By::Id("save")).await {
            save_button.click().await.expect("Failed to save changes");
        };
    }
}

pub fn load_alias_file(file: String) -> Alias {
    let contents = match fs::read_to_string(file) {
        Ok(content) => content,
        _ => panic!("Failed to read alias file"),
    };
    let result: Alias = serde_json::from_str(&contents).expect("Could not parse alias file");
    println!("{:?}", result);
    result
}

pub fn load_dns_forwarder_file(file: String) -> Vec<DNSForwarder> {
    let contents = match fs::read_to_string(file) {
        Ok(content) => content,
        _ => panic!("Failed to read dns file"),
    };
    let result: Vec<DNSForwarder> =
        serde_json::from_str(&contents).expect("Could not parse dns file");
    println!("{:?}", result);
    result
}
