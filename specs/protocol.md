# Verita Messaging Protocol.

O protocolo de mensagens do verita é separado em 2 partes. Primeiramente, a nomenclatura.
Como a ideia do verita é que qualquer um possa hospedar a sua própria versão com suas próprias regras, a gente
separa o verita em 3 principais cores: O client, o Server e o Storage. O client é a parte do usuario final da
versão especifica do verita. O server é um servidor ordinário que implementa a parte Servers desse protocolo
e o storage um terceiro server que implementa a parte Storages desse protocolo.

Essa versão principal do verita tem intuito de ser somente o Server, então você, que quer hospedar, pode
simplesmente clonar o verita, e usar o Storage de seu interesse.

## Messaging

No geral, o protocolo roda em cima de QUIC, sem necessáriamente rodar com nenhuma versão de HTTP, mas no caso
o envio de mensagens é feito via Cap N Proto usando a sub seção Transmission na seção Servers

## Servers Session.
Essa seção se propõe a falar sobre a comunicação client <-> server. Essa parte do protocolo é uma subseção do protocolo por completo que funciona somente
entre client e server.
Nota: Note que o projeto vai crescendo com o tempo, e consequentemente,features e mais features vão sendo adicionadas
e com isso, o protocolo precisa ser revisado e refeito.

Como a ideia que rodeia o verita é de que, qualquer um pode hospedar sua propria versão e acaba virando o centralizador de tudo, referente à ela
isso tem de ser separado nessas etapas.
Para a conexão de um client ser estabelecida, é necessário que o host, em sua versão própria crie as credenciais.
O handshake é feito via Mtls, então qualquer client que se conecte, precisa vir do client que o host prover. Em termos práticos,
se João cria uma versão dele e Pedro cria uma versão dele, ambos precisam criar chaves pra suas respectivas versões, e botar a chave
pública no client pra quando se conectar com o server. Caso dê erro, então é compreendido que você está usando um client não oficial do provedor.

A criação de um usuário é feito então usando 2 coisas, seu ID de criação e sua senha. Como nenhum email é requerido, o sistema de login
é feito via ID, então salvar o ID de alguma maneira é a forma correta de fazer essa conexão. Posteriormente, na seção de Intercomunicação, é dito
como que a comunicação entre diferentes hosts é feita.
Um login então é feito da seguinte forma: Username + Senha, e então um ID é automáticamente atribuido.
Note: Modelo de return tá sendo baseado no de Go

Register {
  username: String,
  password: String //hash
} -> Returns LoginResponse. Registra e loga automaticamente

Login {
  id: UserID,
  password: String //hash
} -> Returns LoginResponse, Error;

LoginResponse {
  sessionToken: UserToken,
  data: UserData //usado apenas no client
}
UserData {
  username: String,
}

UserToken {
  id: Uint64,
  expiry: Uint64, //Se time::now() > expiry, então token invalido
  nonce: Uint64,
  hmac: uint8[32], 
}

A senha no client é descriptografada no client, obviamente, mas no server é sempre esperado que ela siga criptografia de Argon2. Então a senha
que o servidor recebe sempre é esperado para ser Argon2 com Salt de 16bytes
Quando o user loga, é retornado LoginResponse, que contém informações quanto o token de acesso da seção e a informação do user.
O token então tem as informações do Id, até quando o token é valido, um Nonce aleatorio, e um Hmac pra validações pra verificar se o token é valido ou não.
Pelo motivo de precisar de um HMAC, o servidor precisa usar uma chave privada pra verificar essas questões. Isso não é papel desse protocolo, então escolha o que achar melhor

### Chats
The sending of messages is made via Conversations, interactions peer to peer. In groups, the so called, Servers, it's made via 1:N, following the given structs on cap n proto:

PrivateMessage {
  messageId: Uint64,
  roomId: Uint128, min(PublicID) | max(PublicID)
  content: Bytes, //hashed data with the shared key
}

## Storages

Como a ideia é que a parte de storage seja escolhida unicamente pelo host, a parte de storage de conteúdo é feita com intuito de ser burra de propósito.
Por padrão, o storage recebe comandos de o que salvar. Essas coisas vêm em sua maioria criptografadas, pra que ninguém consiga externo ver. Isso então é feito com uma chave derivada, usando Ed25519.
O modelo é simples, o server e o storage contém chaves publicas e privadas, enviam para si as publicas, e tiram uma derivada, a partir disso, o conteúdo é enviado. Todo tipo de conteúdo é internamente um amontoado de bytes
a única questão é que o server sabe como ler esses conteúdos.
Quando enviado um conteudo server -> storage, esse conteúdo chega no storage criptografado.
Quando o server pede um conteúdo pro storage, esse conteúdo vem criptografado e o server envia pro client descriptografado.

O storage tem de ter a opção de salvar conteúdos binários. A forma que isso é feito não é papel desse protocolo, mas todo conteúdo binário tem um handle, que é por ele que é pedido as coisas.
Quando um usuário loga, existe a opção de salvar uma imagem posteriormente, isso então é feito e retornado o handle da imagem. Algo como:

BinaryId {
  handle: Uint64
}

## Error Handling

O error handling vai ser feito por meio de uma struct em especifico, com informações sobre o que ocorreu internamente. A struct pode ser definida como o seguinte:

enum VeritaErrorCode {
  ...
}

struct VeritaError {
  code: VeritaErrorCode,
  timestamp: Utc,
  details: Option<String>
}

type VeritaResponseStatus<T> = Result<T, VeritaError>;
