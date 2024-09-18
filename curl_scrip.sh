# usar timeout Xs para colocar um tempo de requisições
dir="./test_source"
values=("/" $(find $dir -type f | sed "s|^$dir||"))  # Substitua pelos caminhos desejados
iteration=1
while true; do
    random_value=${values[$RANDOM % ${#values[@]}]}  # Seleciona um valor aleatório do vetor
    curl "localhost:8000$random_value" > /dev/null
    echo "Iteração: $iteration"
    ((iteration++))
    sleep 0.2
done
