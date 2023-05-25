
# Interfacing

## Checking evidences


yq ea '[select(file_index == 0)|.Sn*.text] - [select(file_index == 1)|.[]]' examples/example.gsn.yaml localtests/reference.yaml   

## MDG XML