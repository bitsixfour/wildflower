{ self }:
{ config, lib, pkgs, ... }:

let
  cfg = config.services.wildflower;
in
{
  options.services.wildflower = {
    enable = lib.mkEnableOption "wildflower MPD server";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.system}.default;
      description = "Wildflower package to run.";
    };

    createUser = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Create a dedicated system user.";
    };

    user = lib.mkOption {
      type = lib.types.str;
      default = "wildflower";
      description = "Service user and group.";
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = "wildflower";
      description = "Service group.";
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Optional systemd environment file.";
    };

    url = lib.mkOption {
      type = lib.types.str;
      default = "http://192.168.1.20";
      description = "Navidrome server URL.";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 6600;
      description = "MPD listener port.";
    };

    password = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "Navidrome password.";
    };

    usrname = lib.mkOption {
      type = lib.types.str;
      default = "nix";
      description = "Navidrome username.";
    };
  };

  config = lib.mkIf cfg.enable {
    users.groups = lib.mkIf cfg.createUser {
      "${cfg.group}" = { };
    };

    users.users = lib.mkIf cfg.createUser {
      "${cfg.user}" = {
        isSystemUser = true;
        group = cfg.group;
        home = "/var/lib/wildflower";
        createHome = true;
      };
    };

    systemd.services.wildflower = {
      description = "mpd socket";
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];

      environment = {
        MPD_PASS = cfg.password;
        MPD_USER = cfg.usrname;
        MPD_PORT = toString cfg.port;
        MPD_HOST = cfg.url;
      };

      serviceConfig =
        {
          ExecStart = lib.getExe' cfg.package "mpdnavi";
          User = cfg.user;
          Group = cfg.group;
          Restart = "always";
          RestartSec = "10s";
          StateDirectory = "wildflower";
          WorkingDirectory = "/var/lib/wildflower";
        }
        // lib.optionalAttrs (cfg.environmentFile != null) {
          EnvironmentFile = [ cfg.environmentFile ];
        }
        ;
    };
  };
}
