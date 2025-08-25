{
  self,
  lib,
  ...
}: {
  perSystem = {
    pkgs,
    craneLib,
    ...
  }: {
    packages = rec {
      default = discord-presence;
      discord-presence = import ./discord-presence.nix {
        inherit pkgs craneLib self lib;
      };
    };
  };
}
