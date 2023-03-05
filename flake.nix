{
  description = "A pure rust ZMTP implementation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  nixConfig = {
    substituters = [
      "https://francois-caddet.cachix.org"
    ];
    trusted-public-keys = [
      "francois-caddet.cachix.org-1:WYf/RzhEA7GWBOo623fwh9LqXyOQrrZVide6P15GlmQ="
    ];
  };

  outputs = { self, nixpkgs, crane, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        inherit (pkgs) lib;

        craneLib = crane.lib.${system};

        stdout-sink-src = craneLib.cleanCargoSource (pkgs.fetchCrate {
          pname = "stdout-sink";
          version = "0.3.1";
          hash = "sha256-2ST/7NBh/a5qVEjDGkjUolwvOt8HTdVI0U3mihYs+LE=";
        });
        src = craneLib.cleanCargoSource ./.;

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;

          buildInputs = [
            # Add additional build inputs here
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the stdout-sink crate
        stdout-sink = craneLib.buildPackage {
          nativeBuildInputs = with pkgs; [
            zeromq
            pkg-config
          ];
          src = stdout-sink-src;
          doCheck = false; # disable tests because they are not isolated
        };

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        zmtp-rs = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit zmtp-rs;

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          zmtp-rs-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          zmtp-rs-doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
          });

          # Check formatting
          zmtp-rs-fmt = craneLib.cargoFmt {
            inherit src;
          };

          # Audit dependencies
          zmtp-rs-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `zmtp-rs` if you do not want
          # the tests to run twice
          zmtp-rs-nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
        } // lib.optionalAttrs (system == "x86_64-linux") {
          # NB: cargo-tarpaulin only supports x86_64 systems
          # Check code coverage (note: this will not upload coverage anywhere)
          zmtp-rs-coverage = craneLib.cargoTarpaulin (commonArgs // {
            inherit cargoArtifacts;
          });
        };

        packages.default = zmtp-rs;

        apps.default = flake-utils.lib.mkApp {
          drv = zmtp-rs;
          name = "stdin-source";
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          # Extra inputs
          buildInputs = [
            stdout-sink
          ];

          # Extra inputs can be added here
          nativeBuildInputs = with pkgs; [
            cargo
            cargo-edit
            rustfmt
            rustc
            nixpkgs-fmt
          ];
        };
      });
}
