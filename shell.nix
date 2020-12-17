{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  name = "rust-shell";
  nativeBuildInputs = with pkgs; [
    unstable.rust-analyzer
    cargo-watch
    ((rustChannelOf {
      rustToolchain = ./rust-toolchain;
      sha256 = "n6I5wNZmWfYsVKO/abfX0I1GfQOH0tFL+E1d5uaVMgM=";
    }).rust)
  ];
}
