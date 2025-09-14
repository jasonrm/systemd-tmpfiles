{
  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    naersk,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system}.appendOverlays [
        rust-overlay.overlays.default
      ];
      rustFromFile = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-src"];
      };
      nativeBuildInputs = [pkgs.flip-link];
      naerskLib = pkgs.callPackage naersk {
        rustc = rustFromFile;
        cargo = rustFromFile;
      };
    in {
      defaultPackage = naerskLib.buildPackage {
        pname = "systemd-tmpfile";
        root = ./.;
        nativeBuildInputs = nativeBuildInputs;
      };

      defaultApp = flake-utils.lib.mkApp {
        drv = self.defaultPackage."${system}";
      };

      devShell = with pkgs;
        mkShell {
          nativeBuildInputs = [rustFromFile] ++ nativeBuildInputs;

          packages = with pkgs; [
            bashInteractive
            rustfmt
            cargo-flamegraph
          ];

          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
    });
}
