_: {
  perSystem = {
    self',
    pkgs,
    lib,
    ...
  }: {
    devShells.default = pkgs.mkShell rec {
      name = "shell";
      inputsFrom = [
        self'.packages.default
      ];
      packages = with pkgs; [
        rust-analyzer
      ];
      LD_LIBRARY_PATH = "${lib.makeLibraryPath [pkgs.wayland pkgs.libxkbcommon pkgs.vulkan-loader]}";
    };
  };
}
