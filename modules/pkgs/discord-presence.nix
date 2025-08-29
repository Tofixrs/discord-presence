{
  craneLib,
  pkgs,
  self,
  ...
}:
craneLib.buildPackage {
  pname = "discord-presence";
  src = craneLib.cleanCargoSource ../..;
  version = "git-${toString (self.shortRev or self.dirtyShortRev or self.lastModified or "unknown")}";
  strictDeps = true;

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
  buildInputs = with pkgs; [
    wayland
    vulkan-loader
    libxkbcommon
    libappindicator-gtk3
    xdotool
    zenity
  ];
}
