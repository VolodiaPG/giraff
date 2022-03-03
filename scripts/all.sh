files=(*.yml)
total=${#files[@]}
i=0
for file in "${files[@]}"; do
    i=$((i+1))

    faas-cli up -f $file \
        && notify-send FaaS "[$i/$total] done $file" \
        || notify-send FaaS "[$i/$total] failed $file"
done