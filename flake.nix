{
  description = "meow_auth - unauthenticated user goes in, authenticated user goes out";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        craneLib = crane.mkLib pkgs;

        # Common arguments can be set here to avoid repeating them later
        # Note: changes here will rebuild all dependency crates
        commonArgs = {
          src = craneLib.path ./.;
          strictDeps = true;

          SQLX_OFFLINE = true;
          CARGO_PROFILE = "dist";
          RUSTFLAGS = "-C link-arg=-fuse-ld=mold";

          nativeBuildInputs = with pkgs; [
            perl
            pkg-config
            mold-wrapped
            # Add additional build inputs here
          ];
          buildInputs = [
            # Add additional build inputs here
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];
        };

        meow_auth = craneLib.buildPackage (
          commonArgs
          // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;

            # Additional environment variables or build phases/hooks can be set
            # here *without* rebuilding all dependency crates
            # MY_CUSTOM_VAR = "some value";
          }
        );
      in
      {
        # checks = {
        #   inherit meow_auth;
        # };

        packages.default = meow_auth;

        apps.default = flake-utils.lib.mkApp {
          drv = meow_auth;
        };

        # devShells.default = craneLib.devShell {
        #   # Inherit inputs from checks.
        #   checks = self.checks.${system};

        #   # Additional dev-shell environment variables can be set directly
        #   # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

        #   # Extra inputs can be added here; cargo and rustc are provided by default.
        #   packages = [
        #     # pkgs.ripgrep
        #   ];
        # };
      }
    );
}
