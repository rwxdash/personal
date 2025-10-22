#!/bin/bash -ex

# USAGE:
#   ssm cp sre-deploy-instance lab
alias ssm="ssm_connect"
ssm_connect() {
  STAGE=${3%\-*}
  AWS_PROFILE="$1-$STAGE"
  TARGET_EC2_INSTANCE_NAME="$2-$3"
  TOKEN="$4"

  if [[ -n $TOKEN ]]; then
    echo "> Token given! Getting AWS Account ID..."
    ACCOUNT_ID=$(aws sts get-caller-identity | jq -r '.Account')

    echo "> Getting MFA device serial number..."
    MFA_DEVICE_SERIAL_NUMBER=$(aws iam list-mfa-devices | jq -r '.MFADevices[0].SerialNumber')

    echo "> Generating temporary credentials using MFA token..."
    TMP_CREDS="$(aws sts get-session-token --serial-number $MFA_DEVICE_SERIAL_NUMBER --token-code $TOKEN 2>&1)"; CRED_RETURN_CODE=$?;

    if [[ "$CRED_RETURN_CODE" -ne "0" ]]
    then
      echo "> MultiFactorAuthentication failed with invalid MFA one time pass code."
      return 1
    fi

    # EXPIRATION=$(echo $TMP_CREDS | jq -r '.Credentials.Expiration')
    echo "> Setting temporary credentials..."
    export AWS_ACCESS_KEY_ID=$(echo $TMP_CREDS | jq -r '.Credentials.AccessKeyId')
    export AWS_SECRET_ACCESS_KEY=$(echo $TMP_CREDS | jq -r '.Credentials.SecretAccessKey')
    export AWS_SESSION_TOKEN=$(echo $TMP_CREDS | jq -r '.Credentials.SessionToken')
  fi

  if [[ -z $1 || -z $2 || -z $3 ]]
  then
      echo "> AWS_PROFILE (\$1), TARGET_EC2_INSTANCE_NAME (\$2), and STAGE (\$3) arguments must be given!"
      return 1
  else
      echo "> Got AWS_PROFILE: $AWS_PROFILE and TARGET_EC2_INSTANCE_NAME: $TARGET_EC2_INSTANCE_NAME"
      echo "> Fetching target instance ID..."
      TARGET_EC2_INSTANCE_ID=$(aws ec2 describe-instances --region us-west-2 \
        --filters \
                Name=instance-state-name,Values=running \
                Name=tag:Name,Values=$2-$3\
        --query "Reservations[*].Instances[*].InstanceId" \
        --output text)

      if [[ -z $TARGET_EC2_INSTANCE_ID || $TARGET_EC2_INSTANCE_ID == null ]]
      then
          echo "> Couldn't find the instance $TARGET_EC2_INSTANCE_NAME at $AWS_PROFILE"
          return 1
      fi

      echo "> Establishing connection using SSM..."
      aws ssm start-session --target ${TARGET_EC2_INSTANCE_ID} --region us-west-2
  fi

  unset AWS_ACCESS_KEY_ID
  unset AWS_SECRET_ACCESS_KEY
  unset AWS_SESSION_TOKEN
}
