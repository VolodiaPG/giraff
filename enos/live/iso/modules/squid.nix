{
  services.squid.enable = true;
  services.squid.extraConfig = ''
    http_access allow all
    server_persistent_connections off
  '';
}
