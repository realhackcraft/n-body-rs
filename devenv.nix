{ pkgs, lib, config, inputs, ... }:


let
  pkgs-unstable = import inputs.nixpkgs-unstable { system = pkgs.stdenv.system; };
in
{
  languages.rust = {
    channel = "nightly";
    version = "2025-08-23";
    components = [
      "cargo"
      "rust-src"
      "rustc"
      "rust-analyzer"
      "rustfmt"
      "clippy"
      "llvm-tools-preview"
      "rustc-codegen-cranelift-preview"
    ];
    enable = true;
    targets = [
      "x86_64-unknown-linux-gnu"
      "x86_64-pc-windows-msvc"
      "aarch64-apple-darwin"
    ];
  };
  # https://devenv.sh/basics/
  env.GREET = "devenv";
  env.DYLD_LIBRARY_PATH = "${config.languages.rust.toolchainPackage}/lib/rustlib/aarch64-apple-darwin/lib/";
  env.LD_LIBRARY_PATH = "${config.languages.rust.toolchainPackage}/lib/rustlib/aarch64-apple-darwin/lib/";

  # https://devenv.sh/packages/

  packages = [
    pkgs.git
    pkgs.mdbook
    pkgs.mdbook-alerts
    pkgs-unstable.cargo-xwin
  ];

  # https://devenv.sh/languages/
  # languages.rust.enable = true;

  # https://devenv.sh/processes/
  # processes.cargo-watch.exec = "cargo-watch";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
  scripts.hello.exec = ''
    echo hello from $GREET
  '';

  enterShell = ''
    hello
    git --version
  '';

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
  '';

  # https://devenv.sh/pre-commit-hooks/
  # pre-commit.hooks.shellcheck.enable = true;

  # See full reference at https://devenv.sh/reference/options/
}
