#!/bin/bash -ex

# Path to PFX certificate
PFX_PATH=$1
# Path to the file where the cert password resides in plaintext
PASSWORD_PATH=$2

# Prepare output directory
mkdir -p ./out

openssl pkcs12 -in "$PFX_PATH" -nocerts -nodes -passin file:"$PASSWORD_PATH" | sed -ne '/-BEGIN PRIVATE KEY-/,/-END PRIVATE KEY-/p' > ./out/clientcert.key
openssl pkcs12 -in "$PFX_PATH" -clcerts -nokeys -passin file:"$PASSWORD_PATH"  | sed -ne '/-BEGIN CERTIFICATE-/,/-END CERTIFICATE-/p' > ./out/clientcert.cer
openssl pkcs12 -in "$PFX_PATH" -cacerts -nokeys -chain -passin file:"$PASSWORD_PATH" | sed -ne '/-BEGIN CERTIFICATE-/,/-END CERTIFICATE-/p' > ./out/cacerts.cer

# Godaddy CA Issuer
# Not needed anymore since our certs are GoDaddy commercial issued
# CA_ISSUER=http://certificates.godaddy.com/repository/gdig2.crt
# curl $CA_ISSUER --output ./out/intermediate.crt
# openssl x509 -inform DER -in ./out/intermediate.crt -out ./out/intermediate.pem -text
