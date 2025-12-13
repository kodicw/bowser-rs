{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  packages = with pkgs; [
    undetected-chromedriver
    mitmproxy
    chromium
    google-chrome
  ];

  cachix.enable = false;
  languages.rust.enable = true;

  languages.python = {
    uv.enable = true;
    enable = true;
  };

  processes = {
    "proxy".exec = "mitmweb";
    "webdriver".exec = "undetected-chromedriver --port=41435";
  };

  containers."webdriver" = {
    name = "webdriver";
    startupCommand = config.processes.driver.exec;
  };
}
