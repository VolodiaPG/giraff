#!/usr/bin/env -S bash -i
exec ssh -tt nancy.grid5000.fr -- bash -i <<-"__SSH__"
    exec ssh -tt root@10.144.4.2 -- bash -i <<-"__SSH_INNER__"
        cd /home/enos 
        source env.source
        exec podman exec -it $(podman ps -q | head -n1) bash --init-file <(echo 'source \"$HOME/.bashrc\"; source env.source;  exec bash -i') -i
__SSH_INNER__
__SSH__