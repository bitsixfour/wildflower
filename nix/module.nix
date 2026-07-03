{ self }:
{ config, lib, pkgs, ... }:

let
  cfg = config.services.wildflower;
in
{
  options.services.wildflower = {
    enable = lib.mkEnableOption "mpd nix thing??????";


    url = lib.mkOption {
      type = lib.types.str;
      default = "http://192.168.1.20";
      description = "heckin' url for your server. ex: http://192.168.1.67:6767";
    };
    
    port = lib.mkOption {
      type = lib.types.int;
      default = 6600;
      description = "port for your mpd. obviously usually 6600 defaulted";
    };

    password = lib.mkOption {
      type = lib.types.str;
      default = "password";
      description = "plaintext stored password. again your navidrome is read only by design";
    };

    usrname = lib.mkOption {
      type = lib.types.str;
      default = "nix";
      description = "username";
    };
/*
    navidrome = {
      baseUrl = lib.mkOption {
        type = lib.types.str;
        default = "http://127.0.0.1:4533";
        description = "Base URL for the Navidrome server.";
      };

      user = lib.mkOption {
        type = lib.types.str;
        default = "nix";
        description = "Navidrome username.";
      };

      password = lib.mkOption {
        type = lib.types.str;
        default = "";
        description = ''
          Navidrome password exposed directly as plain text through
          OBSIDIANFM_NAVIDROME_PASSWORD.
        '';
      };
    };
*/
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

    systemd.services.obsidianfm = {
      description = "mpd socket";
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];

      environment = {
        PASSWORD = cfg.password;
        USERNAME = cfg.usrname;
        PORT = cfg.port;
        URL = cfg.url;
      };

      serviceConfig =
        {
          ExecStart = lib.getExe cfg.package;
          User = cfg.user;
          Group = cfg.group;
          Restart = "always";
          RestartSec = "10s";
          StateDirectory = "obsidianfm";
          WorkingDirectory = "/var/lib/obsidianfm";
        }
        // lib.optionalAttrs (cfg.environmentFile != null) {
          EnvironmentFile = [ cfg.environmentFile ];
        }
        ;
    };
  };
}
