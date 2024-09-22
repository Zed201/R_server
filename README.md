# R_server
Um servidor http criado a apartir do ultimo projeto do livro oficial de rust, basicamente servindo como um http.server módulo do python.

### TODO:
- [X] Verificar possível memory leak do fato de ter várias threads(não consegui só com o valgrind, pois ele da resultado errado, muito exagerado então imagino que esteja errado)(Ainda fica com um memory leak de 3.87k independente de quantas requisições sejam feitas)
- [X] Aprimorar para algo como live server usando WebSocket(estruturar esquema no miro)
- [ ] Comentar melhor o server/mod.rs
- [X] Modificar como está a estrutura de Requets(talvez trocar para um type apenas de Hashmap)
- [X] Melhorar o threadpool, ou adicionar o tokio, ele não ta pegando no pc de casa(Agora só libera 215 bytes)
- [X] Fazer 2 "modos" um live server e outro server normal
- [ ] Melhorar o resto do sistema com o tungstenite
- [ ] Melhorar os argumentos de cli
- [ ] Optimizar uso de memória
- [ ] Melhorar saida do modo live
- [ ] Testar com multiplos html e testar com retirada de "marcadores" dos vistos
- [ ] Fazer alguma logica para atualizar outros arquivos além de html
