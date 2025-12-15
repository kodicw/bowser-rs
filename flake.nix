{
  description = "A Rust application packaged with Nix flakes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
  };

  outputs =
    { self, nixpkgs, ... }@inputs:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];
      eachSystem = nixpkgs.lib.genAttrs supportedSystems;

      pkgs = eachSystem (
        system:
        import nixpkgs {
          inherit system;
        }
      );

    in
    {
      packages = eachSystem (system: {
        default = pkgs.${system}.rustPlatform.buildRustPackage rec {
          pname = "bowser"; # Replace with your app name
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      });
    };
}
