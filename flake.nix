{
  description = "A tui calendar";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;
      
      manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      pname = manifest.name;
      version = manifest.version;   
    in
    {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        inherit pname version;

        meta = {
          name = pname;
          version = version;
          description = manifest.description;
          licence = manifest.license;
          authors = manifest.authors;
        };
       
        src = pkgs.nix-gitignore.gitignoreSource [] ( pkgs.lib.cleanSource ./. );        
        cargoLock.lockFile = ./Cargo.lock;

        cargoSha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        nativeBuildInputs = [ rustToolchain ];
        
        doCheck = true;
      };

      devShell = pkgs.mkShell {
        buildInputs = [
          rustToolchain
        ];

        RUST_BACKTRACE=1;
      };
    });
}
