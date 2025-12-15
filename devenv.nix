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
    zellij
    nushell
    inputs.polarbear.packages."x86_64-linux".nixvim
  ];
    git-hooks.hooks = {
      rustfmt.enable = true;
      clippy.enable = true;
    };

  cachix.enable = false;
  languages.rust = {
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
  enterShell = '''';
}
