#!/bin/bash

if [ -z ${INWX_USER+x} ]; then
	echo "ERROR: Missing env-argument INWX_USER"
	hasErrors=true
fi

if [ -z ${INWX_PASSWD+x} ]; then
	echo "ERROR: Missing env-argument INWX_PASSWD"
	hasErrors=true
fi

if [ -z ${CERT_EMAIL+x} ]; then
	echo "ERROR: Missing env-argument CERT_EMAIL"
	hasErrors=true
fi

if [ -z ${DOMAINS+x} ]; then
	echo "ERROR: Missing env-argument DOMAINS (i.e. your-domain.com,another-domain.com,*.your-wildcard-domain.com)"
	hasErrors=true
fi

if [ -z ${hasErrors+x} ]; then
	echo "All required params found."
	echo "You can add additional certbot arguments via CMD."
else
	./certbot-auto --help
	exit 1;
fi

printf "$INWX_USER\n$INWX_PASSWD" > /etc/letsencrypt-inwx-cred
chmod 600 /etc/letsencrypt-inwx-cred

set -x
./certbot-auto \
	certonly \
	--non-interactive \
	--agree-tos \
	--preferred-challenges=dns \
	--server https://acme-v02.api.letsencrypt.org/directory \
	--manual \
	--manual-auth-hook /usr/lib/letsencrypt-inwx/certbot-inwx-auth \
	--manual-cleanup-hook /usr/lib/letsencrypt-inwx/certbot-inwx-cleanup \
	--manual-public-ip-logging-ok \
	--email $CERT_EMAIL \
	-d $DOMAINS \
	$@
set +x
rm /etc/letsencrypt-inwx-cred