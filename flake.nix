{
  description = "Development shell for my_teams";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        rustSrc = pkgs.rustPlatform.rustLibSrc;
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            clippy
            rust-analyzer
            rustc
            rustfmt
            rustSrc
          ];

          shellHook = ''
            export RUST_SRC_PATH=${rustSrc}

            if [ -n "$LD_LIBRARY_PATH" ]; then
              export LD_LIBRARY_PATH="$PWD/libs/myteams:$LD_LIBRARY_PATH"
            else
              export LD_LIBRARY_PATH="$PWD/libs/myteams"
            fi
          '';
        };
      });
}
