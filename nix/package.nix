{
  lib,
  stdenv,
  rustPlatform,
  installShellFiles,
}:

let
  manifest = (lib.importTOML ../Cargo.toml).package;
in
rustPlatform.buildRustPackage {
  pname = manifest.name;
  version = manifest.version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../LICENSE
      ../build
      ../src
      ../vendor
      ../benches
      ../tests
    ];
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ installShellFiles ];

  doCheck = false;

  postInstall = lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    installShellCompletion --cmd luadot \
      --bash <($out/bin/luadot completions bash) \
      --zsh <($out/bin/luadot completions zsh) \
      --fish <($out/bin/luadot completions fish)
  '';

  meta = {
    description = manifest.description;
    homepage = manifest.repository;
    license = lib.licenses.mit;
    mainProgram = manifest.name;
    platforms = lib.platforms.unix;
  };
}
