files=(*.yml)
total=${#files[@]}
i=0
for file in "${files[@]}"; do
    i=$((i+1))
    faas-cli up -f $file;

    # Try to display the notification, do not fails if not available
    notify-send FaaS "[$i/$total] done $file" || true
done