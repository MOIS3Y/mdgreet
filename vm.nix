{
  pkgs,
  modulesPath,
  self,
  system,
  ...
}:
{
  imports = [
    (modulesPath + "/virtualisation/qemu-vm.nix")
  ];

  config = {
    nixpkgs.hostPlatform = system;
    networking.hostName = "mdgreet-test-vm";

    environment = {
      variables = {
        WLR_RENDERER_ALLOW_SOFTWARE = "1";
        WLR_NO_HARDWARE_CURSORS = "1";
      };
      systemPackages = with pkgs; [
        alacritty
        cage
        dbus
      ];
      etc."greetd/mdgreet.toml".text = ''
        [appearance]
        label = "NixOS VM Test"
      '';
    };

    # Graphics and Fonts
    hardware.graphics.enable = true;
    fonts.fontconfig.enable = true;

    # Better video driver for QEMU
    virtualisation.qemu.options = [ "-vga virtio" ];

    # Test users
    users.users = {
      alice = {
        isNormalUser = true;
        description = "Alice Liddell";
        password = "password";
        extraGroups = [
          "wheel"
          "video"
        ];
      };
      bob = {
        isNormalUser = true;
        description = "Bob Ross";
        password = "password";
        extraGroups = [
          "video"
        ];
      };
      charlie = {
        isNormalUser = true;
        description = "Charlie Chaplin";
        password = "password";
        extraGroups = [
          "video"
        ];
      };
    };

    # Sessions and system services
    programs.labwc.enable = true;
    programs.niri.enable = true;
    services.accounts-daemon.enable = true;

    # Ensure cache, log and runtime dirs exist and is owned by the greeter
    systemd.tmpfiles.settings."10-mdgreet" = {
      "/var/cache/mdgreet".d = {
        mode = "0755";
        user = "greeter";
        group = "greeter";
      };
      "/var/log/mdgreet".d = {
        mode = "0755";
        user = "greeter";
        group = "greeter";
      };
      "/run/greetd".d = {
        mode = "0700";
        user = "greeter";
        group = "greeter";
      };
    };

    services.greetd = {
      enable = true;
      settings.default_session = {
        command = "${pkgs.writeShellScript "mdgreet-start" ''
          export XDG_RUNTIME_DIR=/run/greetd
          export XDG_CACHE_HOME=/var/cache/mdgreet

          exec ${pkgs.dbus}/bin/dbus-run-session \
            ${pkgs.lib.getExe pkgs.cage} -s -d -- \
            ${pkgs.lib.getExe self.packages.${system}.default}
        ''}";
        user = "greeter";
      };
    };

    # VM Resources
    virtualisation.memorySize = 2048;
    virtualisation.cores = 2;

    services.getty.autologinUser = "root";
    system.stateVersion = "25.11";
  };
}
