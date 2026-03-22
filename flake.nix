{
  description = "SystemVerilog development environment with Verilator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          verilator
          python3
          verible
          gnumake
          gcc
          rustc
          cargo
          stdenv.cc.cc.lib
        ];

        shellHook = ''
          echo "⚡ SystemVerilog + Rust binder development environment"
          echo "Tools: verilator, cargo, rustc, verible"
          echo "Run: make help"
        '';
      };
    };
}
