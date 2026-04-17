{pkgs, ...}:
pkgs.runCommand "rustfmt-check" {
  nativeBuildInputs = [pkgs.rustfmt];
} ''
  cp -r ${../../src} ./src
  rustfmt --check ./src/lib.rs ./src/api.rs ./src/header.rs ./src/userinfo.rs ./src/jeuinfo.rs
  touch $out
''
