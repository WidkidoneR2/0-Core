# INT-043: Attic self-hosted binary cache (local-only).
# Replaces Cachix, whose multi-tenant content-dedup refused to serve our crane deps
# closure. Attic is single-tenant (ours), so it actually stores + serves the closure.
#
# Scope: LOCAL ONLY. Listens on 127.0.0.1:8080 -- serves this machine and a same-host
# VM. No reverse proxy / TLS (not network-exposed). If we later need a real remote
# clean-machine to pull, add nginx+TLS+firewall then (deferred "network Attic").
#
# Secret: JWT RS256 signing secret lives in /etc/atticd.env (root-only, NOT in git,
# NOT in the Nix store which is world-readable). Generate once with:
#   openssl genrsa -traditional 4096 | base64 -w0
# then write:  ATTIC_SERVER_TOKEN_RS256_SECRET_BASE64=<that-value>  into /etc/atticd.env
{ inputs, ... }:
{
  imports = [ inputs.attic.nixosModules.atticd ];

  services.atticd = {
    enable = true;
    environmentFile = "/etc/atticd.env";
    settings = {
      listen = "127.0.0.1:8080";
      # Local single-user cache: default SQLite database + local-file NAR storage
      # (atticd defaults under /var/lib/atticd). No S3/Postgres for local scope.
      # chunking is REQUIRED by atticd's config validation (not optional) -- these are
      # the documented default values (FastCDC content-defined chunking).
      chunking = {
        nar-size-threshold = 65536;   # 64 KiB -- NARs >= this get chunked
        min-size = 16384;             # 16 KiB
        avg-size = 65536;             # 64 KiB
        max-size = 262144;            # 256 KiB
      };
    };
  };
}
