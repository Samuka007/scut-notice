{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        });

        # Common arguments can be set here to avoid repeating them later
        # Note: changes here will rebuild all dependency crates
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          buildInputs = with pkgs; [
            # Runtime dependencies
            openssl
            # Add additional build inputs here
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];
          
          nativeBuildInputs = with pkgs; [
            # Build-time dependencies
            pkg-config
            stdenv.cc  # This provides the C compiler and linker
          ];
        };

        scut-notice = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          # Additional environment variables or build phases/hooks can be set
          # here *without* rebuilding all dependency crates
          # MY_CUSTOM_VAR = "some value";
        });
      in
      {
        checks = {
          inherit scut-notice;
        };

        packages.default = scut-notice;

        apps.default = flake-utils.lib.mkApp {
          drv = scut-notice;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            pkg-config
            openssl
            stdenv.cc
          ];
        };
      });
}
