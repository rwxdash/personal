# Have multiple configuration placed in `~/.kube/conf.d/*` like so;
#
# ```
# ~/.kube
# |-- conf.d
# |   |-- qa-cluster.yaml
# |   `-- prod-cluster.yaml
# `-- config
# ```
#
# Loader for BASH
KUBECONFIG="$HOME/.kube/config"
CONFIG_DIR="$HOME/.kube/conf.d/"
if [ -d "$CONFIG_DIR" ]; then
    CONFIG_FILES=($(find "$CONFIG_DIR" -type f \( -name "*.yaml" -o -name "*.yml" -o -name "*.json" -o -name "*.config" \) | sort))
    for file in "${CONFIG_FILES[@]}"; do
        if [ -z "$KUBECONFIG" ]; then
            KUBECONFIG="${file}"
        else
            KUBECONFIG="${KUBECONFIG}:${file}"
        fi
    done
fi
export KUBECONFIG

# Loader for ZSH
KUBECONFIG="$HOME/.kube/config"
CONFIG_DIR="$HOME/.kube/conf.d/"
CONFIG_FILES=($(find "$CONFIG_DIR" -type f -name "*.yaml" -o -name "*.yml" -o -name "*.json" -o -name "*.config" | sort))
for file in $CONFIG_FILES; do
    if [ -z "$KUBECONFIG" ]; then
        KUBECONFIG="${file}"
    else
        KUBECONFIG="${KUBECONFIG}:${file}"
    fi
done
export KUBECONFIG
