{
  description = "Rust shell flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }: 
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    #rustc_src = fetchTarball {
    #  url = "https://static.rust-lang.org/dist/rustc-1.79.0-src.tar.xz";
    #  sha256 = "1f2qzi421z914gy308s13spzwnhvbhlmyx9l23fhpyhx2y7548rs";
    #};
    #TODO: figure out how to compile this src and use it instead of pkgs.rustc
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
