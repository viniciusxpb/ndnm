# ndnm-brazil: O Backend-for-Frontend (BFF) do NDNM

## 🗣️ O Intermediário Tagarela (V2)

`ndnm-brazil` atua como a ponte de comunicação exclusiva entre o frontend (`ndnm-argos`) e o maestro do backend (`ndnm-hermes`). Ele segue o padrão **Backend-for-Frontend (BFF)**, simplificando a interface para o cliente (`ndnm-argos`) e desacoplando-o das complexidades internas da orquestração feita pelo `ndnm-hermes`.

Sua principal função é traduzir as requisições e eventos do mundo visual do `ndnm-argos` para comandos compreensíveis pelo `ndnm-hermes`, e **repassar as informações detalhadas de configuração e estado** vindas do `ndnm-hermes` para o frontend.

## 🎯 Principais Responsabilidades (Atualizado)

1.  **Interface com `ndnm-argos` (Frontend):**
    * **Comunicação Primária:** Mantém uma conexão **WebSocket** persistente com o `ndnm-argos`.
    * **Envio de Configurações Detalhadas:** Obtém a **estrutura completa e rica** dos nodes disponíveis (incluindo `sections`, `behaviors`, `slot_templates`, `input_fields`) do `ndnm-hermes` e a **envia integralmente** para o `ndnm-argos` via WebSocket. Isso permite que o frontend renderize interfaces de node dinâmicas e complexas.
    * **Repasse de Ações do Usuário:** Recebe comandos do `ndnm-argos` via WebSocket (ex: "executar grafo", "salvar workspace", "disparar refresh de seção dinâmica") e os traduz/repassa para a API do `ndnm-hermes`.
    * **Atualizações de Estado Granulares:** Recebe atualizações de estado detalhadas do `ndnm-hermes` (ex: "node X começou", "node Y concluiu", "erro Z", **"novos slots dinâmicos disponíveis para node W"**) e as retransmite para o `ndnm-argos` via WebSocket para feedback visual em tempo real.
2.  **Interface com `ndnm-hermes` (Maestro):**
    * **Comunicação:** Interage com a API exposta pelo `ndnm-hermes` (ex: REST HTTP, gRPC).
    * **Delegação de Tarefas:** Envia comandos recebidos do frontend para os endpoints apropriados do `ndnm-hermes` (ex: `POST /graphs/run`, `POST /nexus/save`, `GET /nexus/load/{name}`, `POST /nodes/{id}/refresh/{section}`).
    * **Consulta de Informações Detalhadas:** Consulta o `ndnm-hermes` para obter a **estrutura completa e atualizada** dos nodes (`GET /nodes/registry`) sempre que necessário (ex: na conexão inicial do WebSocket).
3.  **Autenticação/Autorização (Futuro):** Permanece como o local ideal para essa lógica.

## 🚫 O Que NÃO Pertence Aqui (Sem Alterações Significativas)

* **Lógica de Negócio dos Nodes:** Continua sendo responsabilidade dos `nodes/*`.
* **Orquestração de Fluxo:** Continua sendo 100% responsabilidade do `ndnm-hermes`.
* **Descoberta de Nodes:** Continua sendo responsabilidade do `ndnm-hermes`. `ndnm-brazil` apenas *consome* essa informação.
* **Gerenciamento de Processos dos Nodes:** Continua sendo responsabilidade do `ndnm-hermes`.
* **Persistência Direta:** Continua intermediando, mas não acessa `nexus/` diretamente.

## 🛠️ Tecnologias Principais (Sem Alterações Significativas)

* **Rust**, **Axum**, **Tokio**, **Serde**, **Reqwest**, **Tokio-Tungstenite**.

**Em resumo (V2):** `ndnm-brazil` é o "intérprete" entre a complexidade visual do `ndnm-argos` e a complexidade orquestral do `ndnm-hermes`. Ele garante que ambos falem a mesma língua (rica em detalhes de configuração e estado) sem precisarem conhecer os dialetos internos um do outro.
