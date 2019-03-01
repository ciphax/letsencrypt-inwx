#!/bin/sh

CONF_CREATED=false

if [ ! -z "$INWX_USER" -a ! -z "$INWX_PASSWD" ]; then
	CONF_CREATED=true
	>&2 echo "\
!!! WARNING !!!
PASSING INWX_USER AND INWX_PASSWD AS ENV VARIABLES IS DEPRECATED AND WILL BE REMOVED IN THE FUTURE!
You should mount a config file into the container instead. See https://github.com/kegato/letsencrypt-inwx for details.
"
	cat << EOF > /etc/letsencrypt-inwx.json
{
	"accounts": [{
		"username": "$INWX_USER",
		"password": "$INWX_PASSWD"
	}]
}
EOF
	chmod 600 /etc/letsencrypt-inwx.json
fi

set -x
certbot -n --agree-tos --server https://acme-v02.api.letsencrypt.org/directory $@
set +x

if [ $CONF_CREATED = true ]; then
	rm /etc/letsencrypt-inwx.json
fi
