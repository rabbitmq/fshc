#!/usr/bin/env nu

let binary = 'fshc'
let os = $env.OS
let src = $env.GITHUB_WORKSPACE
let target = $env.TARGET
let flags = $env.TARGET_RUSTFLAGS

let version = (open Cargo.toml | get package.version)

let dist = $'($env.GITHUB_WORKSPACE)/($binary)-($version)-($target)'
let bin_suffix = if $os == 'windows' { '.exe' } else { '' }
let executable = $'($env.GITHUB_WORKSPACE)/target/($target)/release/($binary)($bin_suffix)'

print $'Packaging ($binary) v($version) for ($target) in ($src)...'

if not ('Cargo.lock' | path exists) {
  cargo generate-lockfile
}

#
# Linux
#

if $os in ['ubuntu', 'ubuntu-latest'] {
  print "Building on Ubuntu..."
  if $target == 'aarch64-unknown-linux-gnu' {
    sudo apt-get install -y gcc-aarch64-linux-gnu
    $env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = 'aarch64-linux-gnu-gcc'
    build-with-cargo $flags
  } else if $target == 'armv7-unknown-linux-gnueabihf' {
    sudo apt-get install pkg-config gcc-arm-linux-gnueabihf -y
    let-env CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER = 'arm-linux-gnueabihf-gcc'
    build-with-cargo $flags
  } else {
    # musl-tools to fix 'Failed to find tool. Is `musl-gcc` installed?'
    sudo apt-get install musl-tools -y
    build-with-cargo $flags
  }
}

if $os in ['fedora', 'fedora-latest'] {
  print "Building on Fedora..."
  if $target == 'aarch64-unknown-linux-gnu' {
    sudo dnf install -y gcc-aarch64-linux-gnu
    $env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = 'aarch64-linux-gnu-gcc'
    build-with-cargo $flags
  } else if $target == 'armv7-unknown-linux-gnueabihf' {
    sudo dnf install pkg-config gcc-arm-linux-gnueabihf -y
    let-env CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER = 'arm-linux-gnueabihf-gcc'
    build-with-cargo $flags
  } else {
    # musl-tools to fix 'Failed to find tool. Is `musl-gcc` installed?'
    sudo dnf install musl-tools -y
    build-with-cargo $flags
  }
}


#
# macOS
#

if $os in ['macos', 'macos-latest'] {
  print "Building on macOS..."
  build-with-cargo $flags
}

#
# Windows
#

if $os in ['windows', 'windows-latest'] {
  print "Building on Windows..."
  cargo rustc --bin $binary --target $target --release
}

#
# Release packaging
#

cd $src
rm -rf $dist
mkdir $dist
print $'Copying release files to ($dist)...'

cp -r LICENSE* $dist
cp -r README* $dist
cp $executable $dist

print "Compiling a release archive..."
if $os in ['ubuntu', 'ubuntu-latest', 'macos', 'macos-latest', 'fedora', 'fedora-latest'] {
  let archive_filename = $'($binary)-($version)-($target).tar.gz'
  print $'Release archive name: ($archive_filename)'
  tar --verbose --directory $src -c --gzip --file $archive_filename $dist
  print $'Release archive at ($archive_filename) is ready'
  echo $'::set-output name=archive::($archive_filename)'
}

def 'build-with-cargo' [ options: string ] {
  if ($options | str trim | is-empty) {
    cargo rustc --bin $binary --target $target --release
  } else {
    cargo rustc --bin $binary --target $target --release $options
  }
}