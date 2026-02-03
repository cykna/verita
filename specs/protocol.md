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

A senha no client é descriptografada, obviamente, mas no server é sempre esperado que ela siga criptografia de Argon2. Então a senha
que o servidor recebe sempre é esperado para ser Argon2 com Salt de 16bytes
Quando o user loga, é retornado LoginResponse, que contém informações quanto o token de acesso da seção e a informação do user.
O token então tem as informações do Id, até quando o token é valido, um Nonce aleatorio, e um Hmac pra validações pra verificar se o token é valido ou não.
Pelo motivo de precisar de um HMAC, o servidor precisa usar uma chave privada pra verificar essas questões. Isso não é papel desse protocolo, então escolha o que achar melhor

### Chats
A seção de Chats define a lógica de interação entre usuários. No Verita, o Chat é E2EE (End-to-End Encrypted) por padrão, tratando o servidor como um retransmissor de mensagens opacas.
1. Tipos de Conversa

As mensagens são categorizadas pelo seu contexto de destino:

    Direct (1:1): Estabelecida através de um handshake inicial de chaves públicas.

    Group (1:N): Baseada em uma Group Key compartilhada entre os membros, permitindo o envio único de binários pesados.

2. O Handshake de Sessão (Key Exchange)

Para garantir que o servidor não tenha acesso às chaves, o protocolo utiliza o conceito de Double Ratchet simplificado ou X25519:

    Identidade: Cada Client possui um par de chaves fixo.

    Session Secret: Na primeira mensagem, o remetente gera um segredo efêmero e o anexa ao cabeçalho da mensagem (MessageHeader).

    Derivação: O destinatário usa sua chave privada para derivar o mesmo segredo. Uma vez estabelecido, o chat pode reutilizar ou rotacionar chaves conforme a necessidade de segurança.

3. Anatomia da Mensagem

As mensagens trafegam via Cap'n Proto para garantir serialização ultrarrápida (zero-copy).
Cap’n Proto

struct ChatMessage {
  header @0 :MessageHeader;
  content @1 :ContentBody;
}

struct MessageHeader {
  conversationId @0 :UInt64;
  senderId @1 :UInt16;       # ID curto para economia de bytes
  nonce @2 :Data;            # Bytes aleatórios para a cifra
  keyMetadata @3 :Data;      # Informações de criptografia (ex: chave efêmera)
}

struct ContentBody {
  union {
    text @0 :Data;           # Texto cifrado e comprimido (Zstd)
    binary @1 :BinaryRef;    # Referência para conteúdo no Storage
    call @2 :CallMetadata;   # Sinais para chamadas de voz/vídeo
  }
}

struct BinaryRef {
  handle @0 :UInt64;         # ID retornado pelo Storage
  key @1 :Data;              # Chave simétrica do arquivo (cifrada para o grupo/user)
  mimeType @2 :Text;         # Dica para o client sobre como renderizar
}

4. Fluxo de Grupos e Links de Convite

Para resolver o problema de "Admin Offline", o protocolo Chat utiliza o Invitee Vault:

    Geração do Link: O Admin cria um link contendo um InviteSecret (não enviado ao servidor).

    Cofre de Boas-Vindas: O Client do Admin criptografa a GroupKey com esse InviteSecret e faz o upload para o Storage.

    Adesão Automática: O novo integrante, ao usar o link, baixa o blob do Storage e extrai a GroupKey localmente.

5. Caching e Persistência no Client

Como o servidor é "burro", o Client é responsável pela integridade do histórico:

    Local Index: O Client mantém um banco de dados local (ex: SQLite) mapeando HandleID para o conteúdo já descriptografado.

    Re-fetch: Se o usuário trocar de dispositivo, o Client pede ao servidor os logs de mensagens e busca os binários no Storage, usando as chaves recuperadas do seu Vault Pessoal.

Resumo Técnico para Implementação:

    Criptografia: ChaCha20-Poly1305 para o corpo das mensagens (mais rápido que AES em processadores sem aceleração de hardware).

    Compressão: Zstd antes da criptografia para reduzir o uso de banda.

    Ordem: As mensagens são enviadas via QUIC Streams independentes para evitar Head-of-Line Blocking (uma mensagem pesada não trava o chat).

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
