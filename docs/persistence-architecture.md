# Arquitetura de Persistencia — Analise e Proposta

## Sumario

1. [Estado Atual](#1-estado-atual)
2. [Pontos de Acoplamento](#2-pontos-de-acoplamento)
3. [Arquitetura Proposta](#3-arquitetura-proposta)
4. [Definicao dos Ports](#4-definicao-dos-ports)
5. [Adapters](#5-adapters)
6. [Models — Conversoes entre Camadas](#6-models--conversoes-entre-camadas)
7. [Erros](#7-erros)
8. [Dependency Injection](#8-dependency-injection)
9. [Trade-offs](#9-trade-offs)
10. [Plano de Migracao](#10-plano-de-migracao)

---

## 1. Estado Atual

### Estrutura de Diretorios

```
verita/
├── Cargo.toml                              # root crate + workspace
├── migration/                              # subcrate de migrations SeaORM
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                          # Migrator
│       ├── main.rs                         # CLI runner
│       └── m20260903_221700_create_topic_subscriptions.rs
├── src/
│   ├── main.rs                             # entry point
│   ├── application.rs                      # Application struct (orchestrator)
│   ├── connection.rs                       # libp2p swarm (P2P layer)
│   ├── bidirectional_channel.rs            # Channel<Msg> generico (flume)
│   └── model/
│       ├── mod.rs                          # pub mod subscriptions
│       └── subscriptions.rs                # SeaORM entity + trait + impl
├── database/
│   └── app.db                              # SQLite database
└── ui/
    └── *.slint                             # Slint UI definitions
```

### Dependencias Principais

| Crate | Versao | Features | Uso |
|-------|--------|----------|-----|
| `sea-orm` | 2.0.2 | `sqlx-sqlite`, `runtime-tokio`, `macros` | ORM |
| `sea-orm-migration` | 2.0.0 | `sqlx-sqlite`, `runtime-tokio-rustls` | Migrations (subcrate) |
| `libp2p` | 0.56.0 | `tokio`, `gossipsub`, `mdns`, `noise`, `tcp`, `yamux`, `quic` | P2P networking |
| `slint` | 1.17.1 | — | GUI nativa |
| `flume` | 0.12.0 | — | Canais bidirecionais |
| `color-eyre` | 0.6.5 | — | Error reporting |
| `tokio` | 1.53.1 | `full` | Async runtime |

### Codigo Nao Compila

A codebase **nao compila atualmente** (5 erros de compilacao):

1. `application.rs:12` — importa `crate::model::Repository` que nao existe em `model/mod.rs`
2. `subscriptions.rs:33` — trait retorna `Vec<subscription::Model>`, impl retorna `Vec<Subscription>` (tipo diferente)
3. `subscriptions.rs:42` — `Subscription::find()` — usa metodos Entity num struct plain
4. `subscriptions.rs:53` — `Subscription::insert()` — idem
5. `subscriptions.rs:24` — `ActiveValue` fora de escopo

O `SubscriptionRepository` trait foi escrito mas **nunca funcionou**. E `application.rs` ignora o trait e usa SeaORM diretamente.

### Fluxo Atual

```
Startup
  │
  ├─ Application::load_database()
  │    └─ Database::connect("sqlite://database/app.db?mode=rwc")
  │    └─ Migrator::up(&database, None)
  │
  ├─ Application::setup()
  │    └─ Entity::find().all(&self.database)          ← query SeaORM direta
  │    └─ Para cada subscription:
  │         └─ join_topic(subscription.id)
  │              ├─ Entity::insert(active_model)      ← query SeaORM direta
  │              └─ Channel::fire(JoinTopic)           ← envia para P2P
  │
  └─ Event loop (Slint UI + P2P swarm)
```

### Operacoes de Banco

| Operacao | Local | Codigo |
|----------|-------|--------|
| SELECT all | `application.rs:50` | `subscriptions::Entity::find().all(&self.database).await?` |
| INSERT | `application.rs:38-42` | `subscriptions::Entity::insert(ActiveModel { ... }).exec(&self.database).await?` |
| UPDATE | — | nao existe |
| DELETE | — | nao existe |
| JOIN | — | nao existe |
| Foreign keys | — | nao existe |

Apenas **1 tabela**, **2 operacoes**, **0 joins**, **0 updates**, **0 deletes**.

### Entidade Unica

```
topic_subscriptions
├── id TEXT PRIMARY KEY    (nome de um topico gossipsub)
```

### Camadas Atuais (conceituais)

| Camada | Arquivos | O que faz |
|--------|----------|-----------|
| UI | `ui/*.slint`, callback em `application.rs` | Interface grafica Slint |
| Application | `application.rs` | Orquestracao: DB + P2P + UI |
| P2P | `connection.rs` | libp2p swarm (gossipsub + mDNS) |
| Domain | `model/subscriptions.rs` (parcial) | Entity `Subscription` (plain) |
| Persistence | `model/subscriptions.rs` (parcial) | SeaORM entity, trait, impl |
| Infrastructure | `migration/` | Schema migrations |
| Cross-cutting | `bidirectional_channel.rs` | Comunicacao entre tasks |

**Problema principal:** Nao ha separacao real entre domain e persistence. O arquivo `model/subscriptions.rs` mistura ambos.

---

## 2. Pontos de Acoplamento

### Onde SeaORM vaza para outras camadas

#### `application.rs` — O pior caso

```rust
// Linha 4: import direto de tipos SeaORM
use sea_orm::{ActiveValue, Database, DatabaseConnection, EntityTrait};

// Linha 21: campo na struct Application
pub struct Application {
    database: DatabaseConnection,   // ← tipo SeaORM no campo
    ...
}

// Linha 38-42: query INSERT usando EntityTrait + ActiveValue
subscriptions::Entity::insert(subscriptions::ActiveModel {
    id: ActiveValue::Set(topic.clone()),
})
.exec(&self.database)
.await?;

// Linha 50: query SELECT usando EntityTrait
subscriptions::Entity::find().all(&self.database).await?;

// Linha 58: criacao da conexao
Database::connect("sqlite://database/app.db?mode=rwc").await?;

// Linha 59: execucao de migrations
Migrator::up(&database, None).await?;
```

**Resumo:** `application.rs` conhece `DatabaseConnection`, `EntityTrait`, `ActiveValue`, `Database`, `Migrator`, `MigratorTrait`, e a Entity `subscriptions` diretamente.

#### `model/subscriptions.rs` — Mistura de dominio e persistence

```rust
// Linha 1: imports de ORM no arquivo que deveria conter o dominio
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, InsertResult};

// Linhas 2-16: Entity SeaORM (com DeriveEntityModel, ActiveModelBehavior)
mod subscription {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "topic_subscriptions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: String,
    }
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

// Linhas 17-19: struct plain duplicado (Subscription)
pub struct Subscription { pub id: String }

// Linhas 21-28: conversao para ActiveModel
impl Into<ActiveModel> for Subscription { ... }

// Linhas 32-37: trait com tipos ORM na interface
pub trait SubscriptionRepository {
    async fn find_all(&self) -> Result<Vec<subscription::Model>, DbErr>;
    async fn insert(&self, subscription: subscription::ActiveModel)
        -> Result<InsertResult<subscription::ActiveModel>, DbErr>;
}
```

**Problemas:**
1. O trait `SubscriptionRepository` expoe `subscription::Model` (tipo ORM) como retorno
2. O trait expoe `subscription::ActiveModel` (tipo ORM) como parametro
3. O trait retorna `DbErr` (tipo ORM) como erro
4. O trait retorna `InsertResult<ActiveModel>` (tipo ORM)
5. O struct `Subscription` (plain) existe mas nunca e usado
6. O `Application` ignora completamente o trait

### Resumo dos Vazamentos

| Tipo SeaORM | Onde vaza | Camada afetada |
|-------------|-----------|----------------|
| `DatabaseConnection` | `application.rs` (campo struct) | Application |
| `EntityTrait` | `application.rs` (import + uso) | Application |
| `ActiveValue` | `application.rs` (import + uso) | Application |
| `Database` | `application.rs` (conexao) | Application |
| `Migrator` / `MigratorTrait` | `application.rs` (import + uso) | Application |
| `DbErr` | `model/subscriptions.rs` (trait signature) | Domain/Repository |
| `InsertResult` | `model/subscriptions.rs` (trait signature) | Domain/Repository |
| `Model` | `model/subscriptions.rs` (trait return type) | Domain |
| `ActiveModel` | `model/subscriptions.rs` (trait param) | Domain |

**Conclusao:** O `application.rs` e o pior afetado — e justamente a camada que deveria ser a mais isolada de detalhes de infraestrutura. O `model/subscriptions.rs` mistura domain e persistence no mesmo arquivo.

---

## 3. Arquitetura Proposta

### Estrutura de Diretorios

```
verita/src/
├── main.rs                                # Composition root
├── application.rs                         # Application service (sem imports de ORM)
├── connection.rs                          # P2P layer (ja isolado, nao muda)
├── bidirectional_channel.rs               # Channel abstraction (ja isolado, nao muda)
│
├── domain/                                # ── DOMINIO ──
│   ├── mod.rs
│   └── subscription.rs                    # Subscription entity (plain Rust)
│
├── repository/                            # ── PORTS ──
│   ├── mod.rs
│   ├── error.rs                           # RepositoryError
│   └── subscription_repository.rs         # trait SubscriptionRepository
│
└── persistence/                           # ── ADAPTERS ──
    ├── mod.rs
    ├── seaorm/                            # Adapter SeaORM
    │   ├── mod.rs
    │   ├── subscription_repository.rs     # impl SubscriptionRepository para SeaORM
    │   └── entities/                      # Entity/Model/ActiveModel (isolados)
    │       ├── mod.rs
    │       └── subscription.rs            # #[derive(DeriveEntityModel)]
    └── memory/                            # Adapter In-Memory (testes/futuro)
        ├── mod.rs
        └── subscription_repository.rs     # impl SubscriptionRepository para Vec
```

### Diagrama de Dependencias

```
                    ┌──────────────────────────────┐
                    │         main.rs               │
                    │    (composition root)          │
                    │                               │
                    │  cria DB connection           │
                    │  cria SeaOrmSubscriptionRepo  │
                    │  passa para Application       │
                    └──────────┬───────────────────┘
                               │
              ┌────────────────┼─────────────────┐
              │                │                  │
              ▼                ▼                  ▼
    ┌─────────────┐  ┌──────────────┐  ┌──────────────────────┐
    │application.rs│  │  repository/ │  │   persistence/       │
    │             │  │              │  │                      │
    │ R: Subscr.  │  │ trait Subsc  │  │ SeaOrmSubscr.        │
    │ Repository  │◄─┤ Repository   │  │ InMemorySubscr.      │
    │             │  │              │  │                      │
    └──────┬──────┘  └──────┬───────┘  └──────┬───────────────┘
           │                │                  │
           ▼                ▼                  ▼
    ┌─────────────┐  ┌──────────────┐  ┌──────────────────────┐
    │  domain/     │  │ repository/  │  │ persistence/seaorm/  │
    │             │  │              │  │                      │
    │ Subscription│  │ depende de   │  │ depende de:          │
    │ (plain)     │  │ domain/      │  │ - domain/            │
    │             │  │              │  │ - repository/         │
    └─────────────┘  └──────────────┘  │ - sea-orm            │
                                       └──────────────────────┘

Legenda:
  ──►  dependencia (importa)
  ◄──  implementa
```

**Regra de ouro:** As setas apontam sempre "para baixo" (ou para a esquerda). Domain nao depende de ninguem. Repository depende de domain. Persistence depende de domain + repository + ORM. Application depende de domain + repository. main.rs orquestra tudo.

### O que NAO muda

- `connection.rs` — ja nao depende de SeaORM, continua isolado
- `bidirectional_channel.rs` — ja nao depende de SeaORM, continua isolado
- `migration/` — continua como subcrate separada (ferramenta de infra, nao pertence ao runtime da app)
- `ui/*.slint` — continua como esta

---

## 4. Definicao dos Ports

### Domain Entity

Arquivo: `src/domain/subscription.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscription {
    pub id: String,
}
```

**Caracteristicas:**
- Zero dependencias externas
- Derive apenas `Debug, Clone, PartialEq, Eq` (sem ORM)
- Nao tem `#[sea_orm(...)]` de nenhum tipo
- Nao tem `Serialize`/`Deserialize` (a menos que necessario para P2P)
- E o tipo que o dominio e a aplicacao conhecem

### Repository Error

Arquivo: `src/repository/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("record not found")]
    NotFound,

    #[error("conflict: record already exists or constraint violation")]
    Conflict,

    #[error("storage unavailable: {0}")]
    Unavailable(String),
}
```

**Por que essas variantes:**

| Variante | Justificativa | Existe hoje? |
|----------|---------------|:------------:|
| `NotFound` | Faz sentido quando houver `find_by_id`. Custo zero antecipar. | Nao (sera usado no futuro) |
| `Conflict` | A tabela usa `id` como PK. Inserir duplicata e um erro real de dominio. | Sim (implicitamente) |
| `Unavailable(String)` | Erro de infra: DB caiu, conexao perdida, timeout. | Sim (DbErr generico) |

**Por que NAO incluir:**
- `Unauthorized`, `Forbidden` — nao fazem sentido neste contexto (P2P, sem auth)
- `Internal(String)` generico demais — `Unavailable(String)` ja cobre falhas de infra
- Qualquer variante que exponha `DbErr` — o adapter traduz internamente

### Repository Trait

Arquivo: `src/repository/subscription_repository.rs`

```rust
use crate::{domain::subscription::Subscription, repository::error::RepositoryError};

pub trait SubscriptionRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Subscription>, RepositoryError>;
    async fn insert(&self, subscription: Subscription) -> Result<(), RepositoryError>;
}
```

**Decisoes justificadas:**

1. **Apenas `find_all` e `insert`**: Sao as unicas operacoes que existem no codigo. Nao crio `find_by_id`, `update`, `delete` que nao existem. Adicionam-se conforme necessidade.

2. **`async fn` direto (sem `#[async_trait]`)**: Rust 2024 (`edition = "2024"` no projeto) suporta `async fn` em traits nativamente. Nao precisa de nenhuma macro externa.

3. **`Send + Sync` bound**: Necessario porque o tokio runtime precisa que futures sejam `Send` para spawned tasks.

4. **Retorna `Result<(), RepositoryError>` no insert, nao `Result<InsertResult<ActiveModel>, DbErr>`**: O dominio nao precisa saber que a insercao retornou o modelo ativo. A operacao inseriu ou falhou.

5. **Parametro e `Subscription` (domain), nao `ActiveModel`**: O dominio nao precisa saber como o ORM representa uma insercao.

6. **Trait definido em `repository/`, nao em `domain/`**: O dominio define *o que e* uma Subscription. O repository define *o que se pode fazer* com Subscriptions. Sao conceitos diferentes.

---

## 5. Adapters

### SeaORM Adapter

Arquivo: `src/persistence/seaorm/subscription_repository.rs`

```rust
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use crate::{
    domain::subscription::Subscription,
    persistence::seaorm::entities::subscription as entity,
    repository::{
        error::RepositoryError,
        subscription_repository::SubscriptionRepository,
    },
};

pub struct SeaOrmSubscriptionRepository {
    db: DatabaseConnection,
}

impl SeaOrmSubscriptionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl SubscriptionRepository for SeaOrmSubscriptionRepository {
    async fn find_all(&self) -> Result<Vec<Subscription>, RepositoryError> {
        let models = entity::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Unavailable(e.to_string()))?;

        Ok(models
            .into_iter()
            .map(|m| Subscription { id: m.id })
            .collect())
    }

    async fn insert(&self, subscription: Subscription) -> Result<(), RepositoryError> {
        let active = entity::ActiveModel {
            id: ActiveValue::Set(subscription.id),
        };
        active.insert(&self.db).await.map_err(|e| match e {
            sea_orm::DbErr::RecordNotInserted | sea_orm::DbErr::RecordNotUpdated => {
                RepositoryError::Conflict
            }
            other => RepositoryError::Unavailable(other.to_string()),
        })?;
        Ok(())
    }
}
```

**Entity isolada dentro do adapter:**

Arquivo: `src/persistence/seaorm/entities/subscription.rs`

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "topic_subscriptions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

**Por que isolar as entities dentro de `persistence/seaorm/entities/`:**
- As entities sao **definicoes de schema**, nao dominio
- Elas contem `DeriveEntityModel`, `ActiveModelBehavior`, etc. — tudo do SeaORM
- Ao isola-las, garantimos que `Model`, `ActiveModel`, `Entity` **nunca** saem do diretorio `persistence/`
- Se amanha trocar de ORM, deleta-se `persistence/seaorm/` inteiro e cria-se `persistence/sqlx/` ou `persistence/diesel/`

**Conversoes no adapter:**

```
find_all:
  Entity::find().all(&db)     →  Vec<Model>         (SeaORM)
  Vec<Model>                  →  Vec<Subscription>   (domain)
  conversao: Subscription { id: m.id }

insert:
  Subscription                →  ActiveModel          (domain → SeaORM)
  conversao: ActiveModel { id: ActiveValue::Set(sub.id) }
  ActiveModel.insert(&db)    →  Result               (SeaORM)
```

### In-Memory Adapter

Arquivo: `src/persistence/memory/subscription_repository.rs`

```rust
use std::sync::Mutex;
use crate::{
    domain::subscription::Subscription,
    repository::{
        error::RepositoryError,
        subscription_repository::SubscriptionRepository,
    },
};

pub struct InMemorySubscriptionRepository {
    subscriptions: Mutex<Vec<Subscription>>,
}

impl InMemorySubscriptionRepository {
    pub fn new() -> Self {
        Self {
            subscriptions: Mutex::new(Vec::new()),
        }
    }
}

impl SubscriptionRepository for InMemorySubscriptionRepository {
    async fn find_all(&self) -> Result<Vec<Subscription>, RepositoryError> {
        Ok(self.subscriptions.lock().unwrap().clone())
    }

    async fn insert(&self, subscription: Subscription) -> Result<(), RepositoryError> {
        let mut subs = self.subscriptions.lock().unwrap();
        if subs.iter().any(|s| s.id == subscription.id) {
            return Err(RepositoryError::Conflict);
        }
        subs.push(subscription);
        Ok(())
    }
}
```

**Utilidades:**
- Testes unitarios de `Application` sem DB
- Validacao de que a abstracao do repository e correta
- Potencialmente util como cache em nodes P2P que nao precisam de DB persistente
- Demonstracao de que a arquitetura funciona

### Outros Adapters Possiveis (futuro)

```
persistence/
├── seaorm/            # SQLite/PostgreSQL via SeaORM
├── memory/            # In-memory (testes)
├── sqlx/              # SQL direto sem ORM (se precisar de controle fino)
├── sled/              # KV store local (nodes leves)
├── sqlite/            # SQLite direto (sem ORM)
└── eventlog/          # Append-only log (P2P event sourcing)
```

A arquitetura permite qualquer uma dessas sem alterar `application.rs` nem `domain/`.

---

## 6. Models — Conversoes entre Camadas

### O que existe em cada camada

```
┌─────────────────────────────────────────────────────────────────┐
│  DOMINIO                                                        │
│  src/domain/subscription.rs                                     │
│                                                                 │
│  #[derive(Debug, Clone, PartialEq, Eq)]                        │
│  pub struct Subscription {                                      │
│      pub id: String,                                            │
│  }                                                              │
│                                                                 │
│  Zero dependencias externas.                                    │
│  Puro Rust.                                                     │
└────────────────────────┬────────────────────────────────────────┘
                         │
          ┌──────────────┴──────────────┐
          │                             │
          ▼                             ▼
┌─────────────────────────┐  ┌────────────────────────────────────┐
│  REPOSITORY PORT         │  │  SEAORM ENTITY (isolado)           │
│  src/repository/         │  │  src/persistence/seaorm/entities/  │
│                          │  │                                    │
│  trait Subscription-     │  │  #[derive(DeriveEntityModel)]      │
│    Repository {          │  │  pub struct Model {                │
│    find_all()            │  │      #[sea_orm(primary_key)]       │
│    insert(Subscription)  │  │      pub id: String,               │
│  }                       │  │  }                                 │
│                          │  │  pub enum ActiveModel { ... }      │
│  Depende apenas de:      │  │  pub enum Entity { ... }           │
│  - domain/               │  │  pub enum Relation {}              │
│  - RepositoryError       │  │                                    │
│                          │  │  Depende de:                       │
└─────────────────────────┘  │  - sea-orm                         │
                              └────────────────────────────────────┘
```

### Tabela Resumo

| Tipo | Camada | Dependencias | Exemplo de uso |
|------|--------|-------------|----------------|
| `Subscription` | Domain | Nenhuma | `Application`, `Repository` trait, handlers |
| `subscription::Model` | Persistence (SeaORM) | `sea-orm` | Dentro do adapter, convertido para `Subscription` |
| `subscription::ActiveModel` | Persistence (SeaORM) | `sea-orm` | Dentro do adapter, criado a partir de `Subscription` |
| `subscription::Entity` | Persistence (SeaORM) | `sea-orm` | Dentro do adapter, para queries |
| `RepositoryError` | Repository | `thiserror` | Retorno dos metodos do repository |

### Duplicacao Justificada

`Subscription { id: String }` e `Model { id: String }` sao estruturalmente identicas. Isso e intencional:

**A favor da duplicacao:**
- Sao conceitos diferentes: "subscription no dominio" vs "linha no banco"
- O banco pode amanha ter `created_at`, `hash`, `version` — nao contaminam o dominio
- O dominio pode amanha ter `is_active`, `topic_type` — nao contaminam o banco
- A conversao e trivial: `Subscription { id: m.id }`
- A fronteira e real: nao ha leaks de tipos

**Contra a duplicacao:**
- Mais codigo (3 linhas de struct + 1 linha de conversao)
- Risco de esquecer de atualizar ambos

**Veredicto:** Duplicar. O custo e minimo (uma struct de 1 campo), o ganho e uma fronteira arquitetural real.

---

## 7. Erros

### Strategy de Erros

```
Application layer                    Repository port                    Persistence adapter
      │                                    │                                    │
      │   color_eyre::Result<T>            │   Result<T, RepositoryError>       │  Result<T, DbErr>
      │   (error generico)                 │   (erros de dominio)               │  (erros do ORM)
      │                                    │                                    │
      │◄───────────────────────────────────┤◄───────────────────────────────────┤
      │          ? operator                │       map_err no adapter           │
```

### Fluxo de Erro

1. **Adapter recebe `DbErr`** do SeaORM
2. **Adapter traduz** para `RepositoryError` via `map_err`
3. **Repository retorna** `Result<T, RepositoryError>` para a application
4. **Application usa `?`** que converte `RepositoryError` para `color_eyre::Report` (via `thiserror` -> `std::error::Error` -> `From<E>` blanket impl)

### Mapeamento DbErr → RepositoryError

```rust
match db_err {
    // Erros de integridade → Conflict
    sea_orm::DbErr::RecordNotInserted => RepositoryError::Conflict,
    sea_orm::DbErr::RecordNotUpdated => RepositoryError::Conflict,

    // Todos os outros → Unavailable (com mensagem)
    other => RepositoryError::Unavailable(other.to_string()),
}
```

**Nota:** O mapeamento exato pode ser refinado conforme novos erros surgirem. O SeaORM tem muitas variantes de `DbErr` — por agora, `Conflict` para violacoes de integridade e `Unavailable` para tudo mais e suficiente.

### Por que NAO expor `DbErr`

- `DbErr` e um tipo do SeaORM — se trocar de ORM, `DbErr` nao existe mais
- O dominio/application nao precisa saber *por que* a operacao falhou internamente
- `RepositoryError` e o contrato: "ou funcionou, ou nao, e aqui estao as razoes que importam para mim"

### O que NAO fazer

```rust
// RUIM — expoe DbErr
pub trait SubscriptionRepository {
    async fn find_all(&self) -> Result<Vec<Subscription>, DbErr>;
}

// RUIM — wrapper generico que nao desacopla
pub enum RepositoryError {
    Db(String),  // "erro do banco" — inutil para o dominio
}

// RUIM — generic error sem semantica
pub enum RepositoryError {
    Other(String),
}
```

---

## 8. Dependency Injection

### Composition Root

Arquivo: `src/main.rs`

```rust
mod domain;
mod repository;
mod persistence;
mod application;
mod bidirectional_channel;
mod connection;

use persistence::seaorm::SeaOrmSubscriptionRepository;
// Para testes:
// use persistence::memory::InMemorySubscriptionRepository;

slint::include_modules!();

// ... ChatBehavior definition ...

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // 1. Criar conexao com o banco
    let db = load_database().await?;

    // 2. Criar o adapter concreto
    let subscription_repo = SeaOrmSubscriptionRepository::new(db);

    // 3. Passar para a application
    Application::run(subscription_repo).await
}

async fn load_database() -> color_eyre::Result<sea_orm::DatabaseConnection> {
    use migration::{Migrator, MigratorTrait};
    let database = sea_orm::Database::connect("sqlite://database/app.db?mode=rwc").await?;
    Migrator::up(&database, None).await?;
    Ok(database)
}
```

### Application Generica

Arquivo: `src/application.rs`

```rust
use crate::{
    domain::subscription::Subscription,
    repository::subscription_repository::SubscriptionRepository,
    // ... outros imports (bidirectional_channel, connection, slint) ...
};

pub struct Application<R: SubscriptionRepository> {
    app: Weak<App>,
    connection_sender: Channel<RequestToConnection>,
    ui_receiver: Channel<ResponseFromUi>,
    subscription_repo: R,
}

impl<R: SubscriptionRepository> Application<R> {
    pub async fn join_topic(&self, topic: String) -> color_eyre::Result<()> {
        self.subscription_repo
            .insert(Subscription { id: topic.clone() })
            .await?;

        self.connection_sender
            .fire(RequestToConnection::JoinTopic(topic))
            .await
    }

    pub async fn setup(&mut self) -> color_eyre::Result<()> {
        for subscription in self.subscription_repo.find_all().await? {
            self.join_topic(subscription.id).await?;
        }
        self.join_topic("hello".into()).await?;
        Ok(())
    }

    pub async fn run(subscription_repo: R) -> color_eyre::Result<()> {
        // ... (canais, swarm, UI — nao muda) ...
        let mut application = Self {
            subscription_repo,
            app: window.as_weak(),
            ui_receiver: ui_response_channel,
            connection_sender: connection_request_channel,
        };
        application.setup().await?;
        // ... (event loop — nao muda) ...
    }
}
```

### Por que Generico e nao Trait Object

| Abordagem | Pro | Contra |
|-----------|-----|--------|
| `R: SubscriptionRepository` (generico) | Zero overhead, monomorfizado, async sem boxing, melhor para tokio | Propaga generico na struct |
| `Box<dyn SubscriptionRepository>` | Flexivel em runtime, sem generico | Overhead de dispatch, precisa `Send + Box<Future>`, mais verboso |

**Para este projeto:** Generico e melhor. `Application` e a unica struct que precisa do generic. Se no futuro surgirem mais repositories, pode-se criar um `Repositories` struct que agrupa tudo:

```rust
pub struct Repositories {
    pub subscriptions: Box<dyn SubscriptionRepository>,
    pub messages: Box<dyn MessageRepository>,
}
```

Por agora, com 1 repository, generico e a escolha mais limpa.

### Por que `load_database` sai de `Application`

Hoje `Application::load_database()` e um metodo estatico que retorna `DatabaseConnection`. Apos a refatoracao:

- `load_database()` retorna `DatabaseConnection` (tipo SeaORM)
- Essa funcao **nao pode ficar** em `Application` porque `Application` nao deve conhecer `DatabaseConnection`
- Move-se para `main.rs` ou para um modulo `infrastructure::database`
- `Application` recebe o repository ja construido

---

## 9. Trade-offs

### 9.1 Repository por Aggregate vs Generico

**Opcao A: Generico**
```rust
trait Repository<T> {
    async fn find_all(&self) -> Result<Vec<T>, RepositoryError>;
    async fn insert(&self, item: T) -> Result<(), RepositoryError>;
}
```

**Opcao B: Por aggregate (escolhida)**
```rust
trait SubscriptionRepository {
    async fn find_all(&self) -> Result<Vec<Subscription>, RepositoryError>;
    async fn insert(&self, subscription: Subscription) -> Result<(), RepositoryError>;
}
```

| Criterio | Generico | Por aggregate |
|----------|:--------:|:-------------:|
| Reflete o dominio | ❌ | ✅ |
| Operacoes semanticas | ❌ find_all de que? | ✅ find_all subscriptions |
| Facil de adicionar operacoes especificas | ❌ precisa de defaults | ✅ so no trait certo |
| Codigo duplicado entre repos | ❌ | ✅ (mas e trivial) |
| Facil de trocar backends | ✅ | ✅ |
| Testabilidade | ✅ | ✅ |

**Veredicto:** Por aggregate. O dominio tem conceitos distintos. Uma `Subscription` nao tem as mesmas operacoes que uma futura `Message` ou `User`.

### 9.2 Duplicacao de Model vs Reutilizacao

**Duplicar (escolhida):**
- `domain::Subscription { id: String }`
- `persistence::seaorm::entities::subscription::Model { id: String }`

**Reutilizar (alternativa):**
- Usar `subscription::Model` como o tipo de dominio

| Criterio | Duplicar | Reutilizar |
|----------|:--------:|:----------:|
| Separacao real | ✅ | ❌ |
| Custo de manutencao | Baixo (1 struct + 1 conversao) | Zero |
| Flexibilidade futura | ✅ dominio e banco evoluem independentes | ❌ acoplamento |
| Complexidade | Levemente mais | Menos |

**Veredicto:** Duplicar. O custo e uma struct de 1 campo e uma linha de conversao. O ganho e que amanha o banco e o dominio evoluem independentes.

### 9.3 Trait no Domain vs Repository

**Opcao A: Trait em `domain/`**

O dominio precisaria conhecer o conceito de "repository" — viola a pureza do dominio.

**Opcao B: Trait em `repository/` (escolhida)**

O dominio e puro (so entidades). O repository e uma porta de infraestrutura.

**Veredicto:** Em `repository/`. O dominio define *entidades e regras de negocio*. O repository define *o contrato de persistencia*.

### 9.4 Async Traits

**Opcao A: `#[async_trait]` macro** — adiciona dependencia e boilerplate de box.

**Opcao B: RPITIT nativo (Rust 2024)** — suportado nativamente, zero overhead.

**Veredicto:** RPITIT nativo. O projeto ja usa `edition = "2024"`.

### 9.5 Testabilidade

Com `InMemorySubscriptionRepository`:

```rust
#[tokio::test]
async fn test_insert_and_find_all() {
    let repo = InMemorySubscriptionRepository::new();
    repo.insert(Subscription { id: "topic-a".into() }).await.unwrap();
    repo.insert(Subscription { id: "topic-b".into() }).await.unwrap();

    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn test_insert_duplicate_returns_conflict() {
    let repo = InMemorySubscriptionRepository::new();
    repo.insert(Subscription { id: "topic-a".into() }).await.unwrap();

    let result = repo.insert(Subscription { id: "topic-a".into() }).await;
    assert!(matches!(result, Err(RepositoryError::Conflict)));
}
```

Sem a abstracao, testar `join_topic` exigiria um DB SQLite real com migrations. Com a abstracao: zero DB, instantaneo, isolado.

### 9.6 Custo de Manutencao

| Componente | Linhas estimadas |
|------------|:----------------:|
| `domain/subscription.rs` | ~3 |
| `repository/error.rs` | ~10 |
| `repository/subscription_repository.rs` | ~6 |
| `persistence/seaorm/subscription_repository.rs` | ~40 |
| `persistence/seaorm/entities/subscription.rs` | ~15 |
| `persistence/memory/subscription_repository.rs` | ~25 |
| Refactor em `application.rs` | ~10 linhas mudadas |
| **Total** | **~109 linhas** |

### 9.7 Risco de Overengineering

**Nivel: BAIXO**

O que NAO e feito:
- ❌ `GenericRepository<T>` com `find_by_id`, `update`, `delete` genericos
- ❌ `Specification` pattern
- ❌ `Unit of Work`
- ❌ `Aggregate Root` base class
- ❌ `Domain Event` system
- ❌ `CQRS` / `Event Sourcing`

O que E feito:
- ✅ 1 struct de dominio
- ✅ 1 trait de repository (2 metodos)
- ✅ 1 enum de erro (3 variantes)
- ✅ 2 adapters (~50 linhas cada)

Isso e a **quantidade minima** de codigo para ter separacao real.

---

## 10. Plano de Migracao

### Pre-condicoes

- O codigo atual **nao compila** (5 erros)
- Ha 1 entity, 2 operacoes, 0 joins
- A refactorizacao pode ser feita como bloco coeso

### Passo 1: Criar camada de dominio

**Arquivos novos:**
- `src/domain/mod.rs`
- `src/domain/subscription.rs`

**Conteudo:** Mover o struct `Subscription` (plain) para `domain/subscription.rs`. Remover qualquer import de ORM.

```rust
// src/domain/subscription.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscription {
    pub id: String,
}
```

```rust
// src/domain/mod.rs
pub mod subscription;
```

**Validacao:** Compila sem imports de ORM.

### Passo 2: Criar a porta (repository trait)

**Arquivos novos:**
- `src/repository/mod.rs`
- `src/repository/error.rs`
- `src/repository/subscription_repository.rs`

**Conteudo:** Definir `RepositoryError` e `trait SubscriptionRepository`.

**Validacao:** Compila sem imports de ORM. Trait usa apenas `domain::Subscription` e `RepositoryError`.

### Passo 3: Criar o adapter SeaORM

**Arquivos novos:**
- `src/persistence/mod.rs`
- `src/persistence/seaorm/mod.rs`
- `src/persistence/seaorm/entities/mod.rs`
- `src/persistence/seaorm/entities/subscription.rs`
- `src/persistence/seaorm/subscription_repository.rs`

**Conteudo:**
- Mover a Entity/Model/ActiveModel definition de `model/subscriptions.rs` para `persistence/seaorm/entities/subscription.rs`
- Implementar `SeaOrmSubscriptionRepository` com conversoes

**Validacao:** O adapter compila. `entities/subscription.rs` tem `sea_orm` imports. `subscription_repository.rs` depende de `domain/` + `repository/` + `entities/`.

### Passo 4: Criar o adapter In-Memory

**Arquivos novos:**
- `src/persistence/memory/mod.rs`
- `src/persistence/memory/subscription_repository.rs`

**Validacao:** Compila sem imports de ORM.

### Passo 5: Refatorar Application

**Mudancas em `application.rs`:**
1. Remover `use sea_orm::{ActiveValue, Database, DatabaseConnection, EntityTrait}`
2. Remover `use migration::{Migrator, MigratorTrait}`
3. Adicionar `use crate::repository::subscription_repository::SubscriptionRepository`
4. Tornar `Application` generico: `Application<R: SubscriptionRepository>`
5. Trocar `database: DatabaseConnection` por `subscription_repo: R`
6. Trocar queries diretas por chamadas ao repository
7. Remover `load_database()` (move para `main.rs`)
8. Atualizar `join_topic()` e `setup()` para usar o repository

### Passo 6: Atualizar composition root

**Mudancas em `main.rs`:**
1. Adicionar modulos: `mod domain; mod repository; mod persistence;`
2. Mover `load_database()` para `main.rs`
3. Criar `SeaOrmSubscriptionRepository` e passar para `Application::run()`

### Passo 7: Remover model/ antigo

**Arquivos para deletar:**
- Deletar `src/model/subscriptions.rs` (substituido por domain/ + persistence/)
- Deletar ou simplificar `src/model/mod.rs`

### Passo 8: Adicionar thiserror

**Mudanca em `Cargo.toml`:**
```toml
thiserror = "2"
```

### Passo 9: Verificacao

```bash
cargo check
cargo test
cargo clippy
```

### Passo 10: Testes

Adicionar testes com `InMemorySubscriptionRepository` para validar a abstracao.

---

## Resumo Final

### O que esta errado hoje

1. `application.rs` usa SeaORM diretamente (queries, conexao, migracao)
2. `model/subscriptions.rs` mistura domain e persistence no mesmo arquivo
3. O `SubscriptionRepository` trait existe mas e morto (nunca chamado) e esta quebrado
4. O struct `Subscription` (plain) existe mas nunca e usado
5. A importacao `crate::model::Repository` nao existe

### O que a proposta resolve

1. **Fronteira arquitetural real** entre dominio e persistencia
2. **Troca de backend** sem alterar `application.rs` nem `domain/`
3. **Testabilidade** via `InMemorySubscriptionRepository`
4. **Tipos limpos** — `Subscription` (domain) e o unico tipo que circula na app
5. **Erros desacoplados** — `RepositoryError` nao expoe `DbErr`

### Criterio de sucesso

Ao final da refactorizacao, `application.rs` e `domain/subscription.rs` devem ter **zero imports de `sea_orm`**. SeaORM deve existir apenas em `persistence/seaorm/`.
