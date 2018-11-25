#!/usr/bin/env bash

if [[ $# -eq 0 ]]; then
  echo "Usage: $0 <private_key.pem>" >&2
  echo "" >&2
  echo "Converts given PEM file into a DER file called \"private_key.der\"." >&2
  exit 1
fi

if [[ ! -f $1 ]]; then
  echo "Could not find file \"$1\"." >&2
  exit 1
fi

if [[ -f private_key.der ]]; then
  echo "private_key.der already exists. Remove it and rerun this command to regenerate it." >&2
  exit 1
fi

openssl rsa -in "$1" -outform DER -out private_key.der
