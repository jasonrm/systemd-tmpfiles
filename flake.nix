{
  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system}.appendOverlays [
          rust-overlay.overlays.default
        ];
        # rustFromFile = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        # rustFromFile = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
        rustFromFile = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
          # targets = [ "aarch64-apple-darwin" ];
        };
        nativeBuildInputs = [ pkgs.flip-link ];
        naerskLib = pkgs.callPackage naersk {
          rustc = rustFromFile;
          cargo = rustFromFile;
        };
      in
      {
        defaultPackage = naerskLib.buildPackage {
          pname = "systemd-tmpfile-rs";
          root = ./.;
          nativeBuildInputs = nativeBuildInputs;
        };

        defaultApp = flake-utils.lib.mkApp {
          drv = self.defaultPackage."${system}";
        };

        devShell = with pkgs; mkShell {
          nativeBuildInputs = [ rustFromFile ] ++ nativeBuildInputs;

          packages = with pkgs; [
            bashInteractive
            rustfmt
          ] ++ (lib.optionals stdenv.isDarwin (with pkgs; with darwin.apple_sdk.frameworks; [
            iconv
            Security
            SystemConfiguration
          ]));

          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      });
}
