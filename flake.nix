{
  description = "Rust shell flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: 
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    packagelist = with pkgs; [
      #rust
      cargo 
      rustc
      rustfmt 
      #bevy
      udev alsa-lib vulkan-loader
      xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # To use the x11 feature
      libxkbcommon wayland # To use the wayland feature
    ];
  in
  {
    devShells.${system}.default = with pkgs; mkShell {
      name = "rust";
      nativeBuildInputs = [ pkg-config ];
      buildInputs = packagelist;
      LD_LIBRARY_PATH = lib.makeLibraryPath packagelist;
      RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
    };
  };
}
